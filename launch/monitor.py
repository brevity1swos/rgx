#!/usr/bin/env python3
"""HN + Reddit comment notification monitor for launch posts.

Polls Hacker News and Reddit for new comments on specified posts and sends
macOS desktop notifications. Tracks seen comment IDs in a local JSON state
file to avoid duplicate alerts.

Usage:
    python launch/monitor.py --hn-id 12345678
    python launch/monitor.py --reddit-url https://reddit.com/r/rust/comments/xxx/...
    python launch/monitor.py --hn-id 12345678 --reddit-url https://... --interval 60

Reddit credentials are read from ../.env.local (REDDIT_CLIENT_ID,
REDDIT_CLIENT_SECRET, REDDIT_USERNAME, REDDIT_PASSWORD).
"""

import argparse
import json
import os
import re
import signal
import subprocess
import sys
import time
from pathlib import Path

import requests

STATE_FILE = Path(__file__).parent / ".monitor_state.json"
ENV_FILE = Path(__file__).resolve().parent.parent.parent / ".env.local"
HN_API = "https://hacker-news.firebaseio.com/v0"
REDDIT_TOKEN_URL = "https://www.reddit.com/api/v1/access_token"
USER_AGENT = "rgx-launch-monitor/0.1 (by /u/rgx_dev)"

# ---------------------------------------------------------------------------
# State persistence
# ---------------------------------------------------------------------------

def load_state() -> dict:
    if STATE_FILE.exists():
        with open(STATE_FILE) as f:
            return json.load(f)
    return {}


def save_state(state: dict) -> None:
    tmp = STATE_FILE.with_suffix(".tmp")
    with open(tmp, "w") as f:
        json.dump(state, f, indent=2)
    tmp.rename(STATE_FILE)


# ---------------------------------------------------------------------------
# macOS notifications
# ---------------------------------------------------------------------------

def notify(title: str, message: str) -> None:
    """Send a macOS notification via osascript."""
    safe_title = title.replace('"', '\\"')
    safe_msg = message.replace('"', '\\"')
    script = (
        f'display notification "{safe_msg}" with title "{safe_title}" sound name "Glass"'
    )
    try:
        subprocess.run(
            ["osascript", "-e", script],
            capture_output=True,
            timeout=5,
        )
    except Exception:
        pass  # notification is best-effort


# ---------------------------------------------------------------------------
# .env.local parsing
# ---------------------------------------------------------------------------

def load_env() -> dict[str, str]:
    env: dict[str, str] = {}
    if not ENV_FILE.exists():
        return env
    with open(ENV_FILE) as f:
        for line in f:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            if "=" in line:
                key, _, value = line.partition("=")
                env[key.strip()] = value.strip()
    return env


# ---------------------------------------------------------------------------
# Hacker News
# ---------------------------------------------------------------------------

def hn_fetch_item(item_id: int) -> dict | None:
    try:
        r = requests.get(f"{HN_API}/item/{item_id}.json", timeout=10)
        r.raise_for_status()
        return r.json()
    except Exception as e:
        print(f"  [HN] Error fetching item {item_id}: {e}")
        return None


def hn_collect_comments(root_id: int) -> list[dict]:
    """BFS traversal of the HN comment tree. Returns all comment items."""
    comments: list[dict] = []
    queue = [root_id]
    while queue:
        item_id = queue.pop(0)
        item = hn_fetch_item(item_id)
        if item is None:
            continue
        if item.get("type") == "comment" and not item.get("deleted") and not item.get("dead"):
            comments.append(item)
        for kid in item.get("kids", []):
            queue.append(kid)
    return comments


def poll_hn(hn_id: int, state: dict, silent: bool) -> int:
    """Poll HN for new comments. Returns count of new comments found."""
    key = f"hn_{hn_id}"
    seen: set[int] = set(state.get(key, []))

    print(f"  [HN] Fetching comments for item {hn_id}...")
    comments = hn_collect_comments(hn_id)
    new_count = 0

    for c in comments:
        cid = c["id"]
        if cid not in seen:
            seen.add(cid)
            new_count += 1
            if not silent:
                author = c.get("by", "anonymous")
                text = c.get("text", "")[:120]
                # Strip HTML tags for notification
                text = re.sub(r"<[^>]+>", "", text)
                notify(
                    f"HN Comment by {author}",
                    text or "(empty comment)",
                )
                print(f"  [HN] New comment by {author}: {text[:80]}...")

    state[key] = list(seen)
    return new_count


# ---------------------------------------------------------------------------
# Reddit
# ---------------------------------------------------------------------------

def reddit_get_token(env: dict[str, str]) -> str | None:
    """Obtain a Reddit OAuth token via password grant."""
    client_id = env.get("REDDIT_CLIENT_ID", "")
    client_secret = env.get("REDDIT_CLIENT_SECRET", "")
    username = env.get("REDDIT_USERNAME", "")
    password = env.get("REDDIT_PASSWORD", "")

    if not all([client_id, client_secret, username, password]):
        print("  [Reddit] Missing credentials in .env.local, skipping Reddit.")
        return None

    try:
        r = requests.post(
            REDDIT_TOKEN_URL,
            auth=(client_id, client_secret),
            data={
                "grant_type": "password",
                "username": username,
                "password": password,
            },
            headers={"User-Agent": USER_AGENT},
            timeout=10,
        )
        r.raise_for_status()
        return r.json().get("access_token")
    except Exception as e:
        print(f"  [Reddit] Auth error: {e}")
        return None


def reddit_extract_post_path(url: str) -> str | None:
    """Extract the API path from a Reddit post URL.

    Accepts URLs like:
        https://www.reddit.com/r/rust/comments/abc123/my_post/
        https://old.reddit.com/r/rust/comments/abc123/my_post
    """
    m = re.search(r"r/(\w+)/comments/(\w+)", url)
    if not m:
        return None
    subreddit, post_id = m.group(1), m.group(2)
    return f"/r/{subreddit}/comments/{post_id}"


def reddit_flatten_comments(node: dict, out: list[dict]) -> None:
    """Recursively flatten Reddit's nested comment structure."""
    if node.get("kind") == "t1":
        data = node.get("data", {})
        out.append(data)
        replies = data.get("replies")
        if isinstance(replies, dict):
            for child in replies.get("data", {}).get("children", []):
                reddit_flatten_comments(child, out)
    elif node.get("kind") == "Listing":
        for child in node.get("data", {}).get("children", []):
            reddit_flatten_comments(child, out)


def poll_reddit(reddit_url: str, token: str, state: dict, silent: bool) -> int:
    """Poll Reddit for new comments. Returns count of new comments found."""
    post_path = reddit_extract_post_path(reddit_url)
    if not post_path:
        print(f"  [Reddit] Could not parse URL: {reddit_url}")
        return 0

    key = f"reddit_{post_path}"
    seen: set[str] = set(state.get(key, []))

    print(f"  [Reddit] Fetching comments for {post_path}...")
    try:
        r = requests.get(
            f"https://oauth.reddit.com{post_path}.json?limit=500&depth=10",
            headers={
                "Authorization": f"Bearer {token}",
                "User-Agent": USER_AGENT,
            },
            timeout=15,
        )
        r.raise_for_status()
        data = r.json()
    except Exception as e:
        print(f"  [Reddit] Fetch error: {e}")
        return 0

    comments: list[dict] = []
    if isinstance(data, list) and len(data) > 1:
        reddit_flatten_comments(data[1], comments)

    new_count = 0
    for c in comments:
        cid = c.get("id", "")
        if not cid or cid in seen:
            continue
        seen.add(cid)
        new_count += 1
        if not silent:
            author = c.get("author", "[deleted]")
            body = (c.get("body") or "")[:120]
            notify(
                f"Reddit Comment by {author}",
                body or "(empty comment)",
            )
            print(f"  [Reddit] New comment by {author}: {body[:80]}...")

    state[key] = list(seen)
    return new_count


# ---------------------------------------------------------------------------
# Main loop
# ---------------------------------------------------------------------------

_shutdown = False


def _handle_sigint(sig, frame):
    global _shutdown
    _shutdown = True


def interruptible_sleep(seconds: int) -> None:
    """Sleep in 1-second increments so Ctrl+C is responsive."""
    for _ in range(seconds):
        if _shutdown:
            break
        time.sleep(1)


def main() -> None:
    parser = argparse.ArgumentParser(
        description="Monitor HN/Reddit posts for new comments and send macOS notifications."
    )
    parser.add_argument("--hn-id", type=int, help="Hacker News item ID to monitor")
    parser.add_argument("--reddit-url", type=str, help="Reddit post URL to monitor")
    parser.add_argument(
        "--interval", type=int, default=60, help="Poll interval in seconds (default: 60)"
    )
    args = parser.parse_args()

    if not args.hn_id and not args.reddit_url:
        parser.error("At least one of --hn-id or --reddit-url is required.")

    signal.signal(signal.SIGINT, _handle_sigint)

    state = load_state()
    env = load_env()

    # Determine if this is a first run (need silent indexing)
    hn_key = f"hn_{args.hn_id}" if args.hn_id else None
    reddit_key = None
    if args.reddit_url:
        post_path = reddit_extract_post_path(args.reddit_url)
        if post_path:
            reddit_key = f"reddit_{post_path}"

    hn_first_run = hn_key and hn_key not in state
    reddit_first_run = reddit_key and reddit_key not in state

    # Acquire Reddit token if needed
    reddit_token = None
    if args.reddit_url:
        reddit_token = reddit_get_token(env)
        if not reddit_token:
            if not args.hn_id:
                print("Error: Reddit auth failed and no HN ID provided. Nothing to monitor.")
                sys.exit(1)
            print("  [Reddit] Will skip Reddit polling (no valid token).")

    # First-run silent indexing
    if hn_first_run or reddit_first_run:
        print("First run detected — indexing existing comments silently...")
        if hn_first_run and args.hn_id:
            count = poll_hn(args.hn_id, state, silent=True)
            print(f"  [HN] Indexed {count} existing comments.")
        if reddit_first_run and args.reddit_url and reddit_token:
            count = poll_reddit(args.reddit_url, reddit_token, state, silent=True)
            print(f"  [Reddit] Indexed {count} existing comments.")
        save_state(state)
        print("Indexing complete. Future runs will notify on new comments.\n")

    print(f"Monitoring started (interval: {args.interval}s). Press Ctrl+C to stop.\n")

    poll_count = 0
    while not _shutdown:
        poll_count += 1
        timestamp = time.strftime("%H:%M:%S")
        print(f"[{timestamp}] Poll #{poll_count}")

        hn_new = 0
        reddit_new = 0

        if args.hn_id:
            try:
                hn_new = poll_hn(args.hn_id, state, silent=False)
            except Exception as e:
                print(f"  [HN] Unexpected error: {e}")

        if args.reddit_url and reddit_token:
            try:
                reddit_new = poll_reddit(args.reddit_url, reddit_token, state, silent=False)
            except Exception as e:
                print(f"  [Reddit] Unexpected error: {e}")

        total_new = hn_new + reddit_new
        if total_new == 0:
            print("  No new comments.\n")
        else:
            print(f"  {total_new} new comment(s) found.\n")

        save_state(state)
        interruptible_sleep(args.interval)

    # Graceful shutdown
    print("\nShutting down — saving state...")
    save_state(state)
    print("State saved. Goodbye!")


if __name__ == "__main__":
    main()
