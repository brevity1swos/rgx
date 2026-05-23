# rgx Launch Playbook

Step-by-step guidance for increasing visibility for rgx.

---

## Status (Updated 2026-05-23)

| Channel | Status | Result |
|---------|--------|--------|
| Show HN | Posted | 0 comments, 11 unique visitors |
| r/rust | **Closed** | AI-generated projects policy. Do not re-attempt. |
| r/commandline | **Closed** | AI disclosure rules + hostile mod climate. Draft in `r_commandline.md`. |
| Terminal Trove | **Listed** | Top referrer |
| awesome-ratatui | **Merged** | PR #248 merged 2026-03-12 |
| awesome-rust | **PR #2522 open** | Submitted 2026-05-23 — pending review |
| This Week in Rust #653 | **PR #8117 open** | Submitted 2026-05-23 — pending review |
| nixpkgs | **Listed (v0.12.1)** | Community-maintained by Cameo007; update issue #523362 filed |
| LinuxLinks | **Listed** | Review published 2026-03-19 |
| AUR | **Listed** | rgx-cli + rgx-cli-bin (community-submitted, not us) |
| users.rust-lang.org | **Pending** | Draft in this file below — post manually |
| Lobste.rs | **Blocked** | Needs invite |

**Current metrics (2026-05-23):** 198 stars, 872 total downloads, 687 recent downloads

---

## Immediate Next Actions

### 1. Post to users.rust-lang.org Showcase

Post the draft below at: https://users.rust-lang.org/c/community/showcase/

**Title:** `rgx — a regex debugger for the terminal (step-through execution, 3 engines, code generation)`

**Body:**

```
I've been working on rgx, a terminal regex debugger written in Rust
using ratatui + crossterm. Just shipped v0.12.3.

**What makes it different from other terminal regex tools:**

The main thing I haven't seen elsewhere is a step-through debugger
(Ctrl+D, PCRE2) — it traces execution with a dual-cursor visualization
(pattern cursor + text cursor moving together), backtracking markers,
and a heatmap mode that shows which parts of the pattern are matched
most often. Useful when a pattern works but you don't understand why,
or when backtracking is making it slow.

Beyond that:
- **3 engines** — Rust `regex` (default), `fancy-regex` (lookaround /
  backrefs), PCRE2. Auto-selects the simplest engine that supports your
  pattern's features.
- **Code generation** (Ctrl+G) — emits ready-to-paste code for Rust,
  Python, JS, Go, Java, C#, PHP, Ruby
- **Generate regex from examples** (Ctrl+X) — grex overlay
- **Live filter mode** — `rgx filter` streams stdin/file through a regex
  TUI; `--json <PATH>` extracts a specific JSONL field to match against
- **Batch/pipeline mode** — `echo "log" | rgx -p '\d+'` with grep-like
  exit codes
- **Test suite mode** — TOML-based CI assertions (`rgx --test`)
- VS Code, Neovim, Zed, tmux integrations

Install:
    cargo install rgx-cli
    brew install brevity1swos/tap/rgx

GitHub: https://github.com/brevity1swos/rgx
```

### 2. Re-record demo GIF

The current `assets/demo.gif` predates the step-through debugger (Ctrl+D),
grex overlay (Ctrl+X), code generation (Ctrl+G), and filter mode. These are
the features that differentiate rgx — the first impression is the most
important conversion moment.

```bash
PATH=$HOME/.cargo/bin:$PATH vhs assets/demo.tape
```

Note: VHS does not support F1 key.

### 3. Lobste.rs

Needs an invite from an existing member. Worth pursuing if you know one.

### 4. dev.to article

**Title:** "Building a regex debugger for the terminal in Rust"

Publish to dev.to, then cross-post link to:
- This Week in Rust (another PR, linking the article instead of the release)
- HN: Show HN
- users.rust-lang.org (update your showcase thread)

---

## Closed / Not Planned

| Channel | Reason |
|---------|--------|
| r/rust | AI-generated projects policy — closed permanently |
| r/commandline | AI disclosure rule + hostile mod climate |
| Hosted community platform | Against zero-hosted principle |
| AI/LLM integration | Not planned (see ROADMAP.md) |

---

## Monitoring

```bash
# Stars
gh api repos/brevity1swos/rgx --jq '.stargazers_count'

# crates.io downloads
curl -s https://crates.io/api/v1/crates/rgx-cli | jq '.crate | {downloads, recent_downloads}'

# Traffic referrers (requires auth)
gh api repos/brevity1swos/rgx/traffic/popular/referrers

# Open PRs and issues
gh pr list --repo brevity1swos/rgx
gh issue list --repo brevity1swos/rgx
```
