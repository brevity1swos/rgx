# rgx Launch Playbook

Step-by-step instructions to increase visibility for rgx. Follow in order — each step builds on the previous.

---

## Launch Status (Updated March 11, 2026)

| Channel | Status | Result |
|---------|--------|--------|
| Show HN | Posted | 0 comments, 11 unique visitors from HN |
| r/rust | **Declined** | Subreddit does not accept AI-generated projects. Do not re-attempt. |
| r/commandline | Posted (v1) | GitHub was 404 when users clicked. Re-post with v2 draft. |
| Terminal Trove | Not yet submitted | |
| awesome-ratatui | PR #248 submitted | Pending review |
| awesome-rust | Not yet | Need stars > 50 or downloads > 2000 |

**Current metrics:** 8 stars, 284 downloads, 68 unique visitors (14-day)

**Blocker:** GitHub Actions is disabled for the account. Re-enable before cutting releases.

---

## Pre-Launch Checklist

Before posting anywhere, verify:

- [x] Demo GIF is up to date (`assets/demo.gif`)
- [x] Screenshot PNG for Terminal Trove (`assets/social-preview.png`)
- [x] README comparison table is current
- [x] README documents pipeline features (`--print`, `--output-pattern`, exit codes)
- [x] GitHub repo description is updated
- [x] crates.io listing is live (`cargo install rgx-cli` works)
- [x] Homebrew formula works (`brew install brevity1swos/tap/rgx`)
- [x] All tests pass (`cargo test --no-default-features`)
- [x] Clippy clean (`cargo clippy --no-default-features -- -D warnings`)
- [ ] **Re-enable GitHub Actions** — currently disabled, blocking CI and releases
- [ ] **Cut v0.7.0 release** — 3 unreleased features on main (recipe library, benchmark mode, Neovim plugin)
- [ ] **Verify demo GIF shows current features** — re-record with `PATH=$HOME/.cargo/bin:$PATH vhs assets/demo.tape` if needed

---

## Next Actions (Priority Order)

### 1. Re-enable GitHub Actions

GitHub Actions is disabled for the account. Go to:
`https://github.com/settings/actions` → enable Actions for the repository.

Once enabled, push any commit to main to trigger:
- CI (tests, clippy, fmt)
- release-plz (will create PR to bump to v0.7.0)

### 2. Cut v0.7.0 Release

After re-enabling Actions, release-plz will auto-create a release PR with:
- feat: add regex recipe library with Ctrl+R picker
- feat: add group extraction, benchmark mode, and Neovim plugin
- refactor: deduplicate code and simplify benchmark logic

Merge the PR → dist workflow builds binaries → v0.7.0 on GitHub Releases + crates.io.

### 3. Re-post to r/commandline

Use the updated draft in [`launch/r_commandline.md`](r_commandline.md) (v2).

**Key changes from v1:**
- Leads with "two modes" framing (interactive TUI + batch/pipeline)
- Concrete pipeline examples front and center
- Mentions recipe library (new since v1)
- Removes items that r/commandline doesn't care about (undo/redo, pattern history)

**When:** After v0.7.0 is released. Tuesday or Wednesday, morning EST.

### 4. Submit to Terminal Trove

Use details from [`launch/terminal_trove.md`](terminal_trove.md). Can be done same day as r/commandline post.

### 5. Follow up on awesome-ratatui PR

Check status of PR #248: https://github.com/ratatui/awesome-ratatui/pull/248

---

## Closed Channels

### r/rust — Do Not Re-attempt

r/rust declined the post. Their policy does not accept AI-generated projects.
This channel is closed. Do not re-submit.

Alternative Rust community options:
- **This Week in Rust** — submit to https://github.com/rust-lang/this-week-in-rust (curated newsletter, not self-post)
- **users.rust-lang.org** — Showcase category: https://users.rust-lang.org/c/community/showcase/
- **Rust community Discord** — #showcase channel

---

## Phase 2: Growth (After initial posts)

### Monitor and Engage

- **Check HN** daily for new comments — respond to all
- **Check Reddit** daily — reply to questions and feedback
- **Check GitHub Issues** — new users may file bugs or feature requests
- **Track stats:**
  ```bash
  gh api repos/brevity1swos/rgx --jq '.stargazers_count'
  curl -s https://crates.io/api/v1/crates/rgx-cli | jq '.crate.downloads'
  ```

### Cross-Post to Other Communities

If r/commandline post gets traction:

- **r/linux** — "I built a terminal regex tester as an alternative to regex101.com"
- **Lobste.rs** — submit the GitHub URL (needs invite or existing account)
- **users.rust-lang.org** — Showcase post
- **This Week in Rust** — PR to the newsletter repo
- **dev.to** — short article: "Building a regex debugger for the terminal in Rust"

### awesome-rust Submission

**Prerequisite:** stars > 50 OR downloads > 2000

When threshold is met, follow [`launch/awesome_rust_draft.md`](awesome_rust_draft.md).

### Package Manager Submissions

**AUR (Arch Linux):**
- Create a PKGBUILD
- regex-tui is on AUR — precedent exists

**Nix:**
- Add package to nixpkgs
- Nix users overlap heavily with target audience

### Create Public GitHub Issues

Create 3-5 roadmap issues to signal active development:
- Performance comparison display mode
- Export/share functionality
- Theme customization
- Shell completions (bash/zsh/fish)

### Blog Post

**Title:** "Building a regex debugger for the terminal in Rust"

Publish to dev.to, then submit to HN and users.rust-lang.org.

---

## Key Metrics

| Metric | How to Check |
|--------|-------------|
| GitHub stars | `gh api repos/brevity1swos/rgx --jq '.stargazers_count'` |
| crates.io downloads | `curl -s https://crates.io/api/v1/crates/rgx-cli \| jq '.crate.downloads'` |
| GitHub traffic | Settings → Traffic on GitHub |
| Referrers | `gh api repos/brevity1swos/rgx/traffic/popular/referrers` |
