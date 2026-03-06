# rgx Launch Playbook

Step-by-step instructions to increase visibility for rgx. Follow in order — each step builds on the previous.

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
- [ ] **Verify demo GIF shows current features** — re-record with `PATH=$HOME/.cargo/bin:$PATH vhs assets/demo.tape` if needed

---

## Phase 1: Launch Day (Do all on the same day)

### Step 1: Post to Hacker News (Show HN)

**When:** Tuesday or Wednesday, 8-10 AM EST (best traffic)

1. Go to https://news.ycombinator.com/submit
2. **Title:** `Show HN: rgx – a terminal regex tester with live matching and 3 engines`
3. **URL:** `https://github.com/brevity1swos/rgx`
4. **Text:** Copy the body from [`launch/show_hn.md`](show_hn.md) (the "Text" section)
5. After posting, note the HN URL for use in Reddit posts

**Tips:**
- Respond to every comment within the first 2 hours
- Be upfront about limitations — HN respects honesty over salesmanship
- If asked "why not just use regex101?" — be honest: regex101 is more capable overall, rgx fills specific gaps (offline, pipelines, engine-specific testing)
- If asked about performance, mention the Criterion benchmarks
- Highlight the pipeline story: `cat log | rgx -p 'ERROR: (.*)' | sort` — HN loves composable Unix tools

### Step 2: Post to r/rust

**When:** Same day as HN, 1-2 hours after

1. Go to https://www.reddit.com/r/rust/submit
2. **Title:** `rgx — a terminal regex tester with 3 engines, built with ratatui + regex-syntax`
3. **Body:** Copy from [`launch/r_rust.md`](r_rust.md)
4. **Flair:** Use "Show" or "Tools & Libraries" flair if available

**Tips:**
- r/rust loves technical details — the ratatui + regex-syntax stack is interesting
- Mention the engine abstraction pattern — Rust devs appreciate clean trait design
- The `--no-default-features` zero-C-dependency story resonates with Rust purists
- Link to the HN discussion if it's getting traction

### Step 3: Post to r/commandline

**When:** Same day, 2-3 hours after HN

1. Go to https://www.reddit.com/r/commandline/submit
2. **Title:** `rgx — a TUI regex tester with live matching, 3 engines, and stdin pipe support`
3. **Body:** Copy from [`launch/r_commandline.md`](r_commandline.md)

**Tips:**
- This audience cares about practical utility, not implementation details
- Lead with pipeline features: `echo "data" | rgx -p '\d+'`, exit codes, `$(rgx -P)` pattern capture
- The comparison table is your strongest asset here

### Step 4: Submit to Terminal Trove

**When:** Same day

1. Go to https://terminaltrove.com/post/
2. Fill in the form using details from [`launch/terminal_trove.md`](terminal_trove.md)
3. Upload `assets/social-preview.png` as the preview image
4. Submit and wait for curator review

---

## Phase 2: First Week (Days 2-7)

### Step 5: Monitor and Engage

- **Check HN** daily for new comments — respond to all
- **Check Reddit** daily — reply to questions and feedback
- **Check GitHub Issues** — new users may file bugs or feature requests
- **Track stars:** `gh api repos/brevity1swos/rgx --jq '.stargazers_count'`
- **Track downloads:** `curl -s https://crates.io/api/v1/crates/rgx-cli | jq '.crate.downloads'`

### Step 6: awesome-ratatui PR Follow-Up

- PR already submitted: https://github.com/ratatui/awesome-ratatui/pull/248
- Check if maintainers have reviewed it
- Respond to any requested changes promptly

### Step 7: Cross-Post to Other Communities

If the initial posts gain traction:

- **r/linux** — "I built a terminal regex debugger as an alternative to regex101.com"
- **Lobste.rs** — submit the GitHub URL (you need an invite or existing account)
- **dev.to** — write a short article: "Building a regex101 clone for the terminal in Rust"
- **Twitter/X** — post the demo GIF with a short description, tag @raboratui and @AhmedSoliman (ratatui maintainer)
- **Mastodon** — post on fosstodon.org or similar tech instances

---

## Phase 3: Growth (Week 2+)

### Step 8: Submit to awesome-rust

**Prerequisite:** Need `stars > 50` OR `downloads > 2000`

Check if ready:
```bash
gh api repos/brevity1swos/rgx --jq '.stargazers_count'
curl -s https://crates.io/api/v1/crates/rgx-cli | jq '.crate.downloads'
```

When threshold is met:
1. Fork `rust-unofficial/awesome-rust`
2. Add to the **Text processing** section (alphabetically):
   ```
   - [brevity1swos/rgx](https://github.com/brevity1swos/rgx) [[rgx-cli](https://crates.io/crates/rgx-cli)] - Terminal regex debugger with real-time matching, 3 engines, capture group highlighting, and plain-English explanations.
   ```
3. Create PR — see [`launch/awesome_rust_draft.md`](awesome_rust_draft.md)

### Step 9: grex Cross-Reference

1. If rgx gains enough traction to be credible, consider opening an issue on [grex](https://github.com/pemistahl/grex) (8K+ stars)
2. Suggest `grex "foo" "bar" | rgx` as a complementary workflow in their README
3. Only do this if rgx has real users — otherwise it reads as self-promotion
4. grex generates regex from examples -> rgx tests/refines them. Natural workflow, but only worth mentioning if both tools' maintainers see mutual value.

### Step 10: Package Manager Submissions

**AUR (Arch Linux):**
- Create a PKGBUILD following https://wiki.archlinux.org/title/Creating_packages
- Submit to https://aur.archlinux.org/
- regex-tui is on AUR, so there's precedent

**Nix:**
- Add a package to nixpkgs: https://github.com/NixOS/nixpkgs
- Follow https://nixos.org/manual/nixpkgs/stable/#sec-package-simple
- Nix users heavily overlap with terminal power users

### Step 11: Create Public GitHub Issues

Create these issues to signal active development and invite contribution:

1. **"Performance comparison mode"** — run same pattern on all engines, show timing
   - Label: `enhancement`, `good first issue`
2. **"Regex recipe library"** — embed common patterns (email, URL, IP, date)
   - Label: `enhancement`
3. **"Export/share functionality"** — copy as formatted text or generate regex101 URL
   - Label: `enhancement`, `good first issue`
4. **"Theme customization"** — user-configured themes via config.toml
   - Label: `enhancement`, `good first issue`

### Step 12: Write a Blog Post

**Title:** "Building a regex101 clone for the terminal in Rust"

**Outline:**
1. Why terminal regex debugging matters
2. Architecture: ratatui + crossterm + multi-engine abstraction
3. The regex-syntax AST walker for plain-English explanations
4. Multi-engine support: one trait, three implementations
5. Lessons learned building TUI apps in Rust

**Publish to:** dev.to, personal blog, then submit to HN and r/rust

---

## Tracking Progress

Use this checklist to track what's done:

```
Launch Day:
  [ ] Show HN posted
  [ ] r/rust posted
  [ ] r/commandline posted
  [ ] Terminal Trove submitted

First Week:
  [ ] Engaged with all HN/Reddit comments
  [ ] awesome-ratatui PR merged
  [ ] Cross-posted to additional communities

Growth:
  [ ] awesome-rust PR submitted (after threshold)
  [ ] grex cross-promotion issue opened
  [ ] AUR package submitted
  [ ] Nix package submitted
  [ ] GitHub issues created
  [ ] Blog post published
```

---

## Key Metrics to Track

| Metric | Where to Check |
|--------|---------------|
| GitHub stars | `gh api repos/brevity1swos/rgx --jq '.stargazers_count'` |
| crates.io downloads | `curl -s https://crates.io/api/v1/crates/rgx-cli \| jq '.crate.downloads'` |
| GitHub traffic | Settings -> Traffic on GitHub |
| HN points | Check the post |
| Reddit upvotes | Check posts |

Note: Star count will likely exceed actual regular users — that's normal for developer tools. Focus on whether the people who try it find it genuinely useful for their workflow rather than chasing star numbers.
