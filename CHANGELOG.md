# Changelog

All notable changes to this project will be documented in this file.

## [0.5.0] - 2026-02-26

### Bug Fixes

- Bounds safety, VecDeque history, config wiring, and code quality
- Fix scroll_to_selected() bounds check and u16 overflow safety
  - Change pattern_history from Vec to VecDeque for O(1) front-removal
  - Add Copy derive to EngineFlags; extract wrap_pattern() to deduplicate
    flag prefix logic in rust_regex.rs and fancy.rs
  - Add named panel constants (PANEL_REGEX, PANEL_TEST, etc.) replacing
    magic numbers; consolidate editor dispatch with closures
  - Expand Settings with flag fields, parse_engine(); make CLI engine/unicode
    optional so config defaults apply; wire settings loading in main
  - Add Unicode edge case tests (emoji, CJK, combining marks), empty
    state tests, invalid capture ref test, and config deserialization tests
- Resolve clippy field_reassign_with_default and add launch monitor
Use struct initialization with ..Default::default() instead of mutable
  field reassignment in config_tests to satisfy clippy on Rust 1.93.

  Also adds HN/Reddit comment notification monitor script and updates
  .gitignore for monitor state file.

### Documentation

- Regenerate demo GIF with current features
- Add syntax highlighting to feature list and bump demo GIF cache


## [0.4.1] - 2026-02-22

### Documentation

- Update demo tape for multi-line input and whitespace visualization
- Add launch post drafts and submission materials
Show HN, r/rust, r/commandline post drafts, Terminal Trove submission
  details, and awesome-rust draft entry (deferred until star/download
  threshold is met). awesome-ratatui PR already submitted ([#248](https://github.com/brevity1swos/rgx/pull/248)).
- Add launch playbook with step-by-step visibility guide


## [0.4.0] - 2026-02-22

### Features

- Pre-launch polish — fix UTF-8 bugs, add whitespace viz, word movement, clipboard timer
- Fix expand_replacement() byte-level `as char` casting that broke on non-ASCII
    replacement templates; rewrite to iterate by char_indices
  - Fix truncate() char boundary panic on multi-byte UTF-8 by using char_indices().nth()
  - Add whitespace visualization toggle (Ctrl+W): spaces as ·, newlines as ↵, tabs as →
  - Add Ctrl+Left/Right word-level cursor movement (move_word_left/move_word_right)
  - Extend clipboard status display from instant dismiss to ~2 seconds (40 tick counter)
  - Add multi-line matching tests (multiline flag, line anchors, dotall) for all engines
  - Update GitHub repo description to mention all v0.3.0 features
  - Update README with new keyboard shortcuts and whitespace visualization feature


## [0.3.0] - 2026-02-22

### Documentation

- Cache-bust demo GIF URL for GitHub CDN

### Features

- Add match detail/clipboard, cheat sheet, history/undo, mouse support
- Undo/redo (Ctrl+Z / Ctrl+Shift+Z) for all editor panels with 500-entry stack
  - Pattern history (Alt+Up/Down) with dedup and 100-entry cap
  - Match selection (Up/Down in matches panel) with >> highlight and capture navigation
  - Copy selected match to clipboard (Ctrl+Y) via arboard with status feedback
  - Context-sensitive 3-page F1 cheat sheet: shortcuts, regex syntax, engine-specific
  - Mouse support: click to focus/position cursor, scroll to navigate panels
  - Extract layout computation for mouse hit-testing (PanelLayout struct)
  - Update status bar hints, README features/shortcuts/comparison, demo assets


## [0.2.0] - 2026-02-22

### Bug Fixes

- Remap help key from ? to F1 so ? can be typed in patterns
The ? key was intercepted by ShowHelp before reaching InsertChar,
  making it impossible to type common regex syntax like (?P<name>...),
  \w+?, (?:...), etc. Remap help to F1 and add UI tests for match
  display rendering and multi-line test strings.
- Prevent subtraction overflow in regex input on zero-size terminals
Use saturating_sub for title truncation width and cursor bounds checks
  to avoid panicking when the render area has zero width or height.

### Features

- Fix named captures, add scrollable panels and multi-line editor
- Fix named capture groups in fancy-regex and PCRE2 engines by using
    capture_names() API instead of hardcoding None
  - Add scrollable match display and explanation panels with focus cycling
    across all 4 panels (Tab), scroll via Up/Down on focused panel
  - Implement multi-line test string editor with Enter for newlines,
    Up/Down cursor navigation, vertical scroll, and line-aware highlighting
  - Grow test string area from 3 to 8 rows for multi-line content
- Add regex pattern syntax highlighting in the input field
Color parentheses, quantifiers, character classes, escapes, anchors,
  and alternation operators using the Catppuccin palette. Walks the
  regex-syntax AST to categorize tokens and builds colored ratatui spans.
  Falls back to plain text on parse failure.
- Add live replace/substitution mode with highlighted preview
Add a replacement input panel between test string and results area,
  enabling real-time substitution preview. Supports $1, ${name}, $0/$&,
  and $$ syntax. Engine-agnostic replacement operates on computed match
  data so it works identically across all three engines.

  - Add ReplaceSegment, ReplaceResult, expand_replacement(), replace_all()
  - Add replace_editor, replace_result state to App with rereplace() chain
  - New ReplaceInput widget (single-line, panel index 2)
  - MatchDisplay renders highlighted preview (green bg for replacements)
  - Layout updated from 4 to 5 panels, Tab cycles all five
  - CLI flag -r/--replacement for initial replacement string
  - 12 new tests (7 unit + 5 integration)


## [0.1.9] - 2026-02-22

### Features

- Automate Homebrew tap publishing on release
- Add publish-homebrew job to dist.yml that pushes formula to
    brevity1swos/homebrew-tap on each release
  - Add tap config to Cargo.toml for cargo-dist
  - Formula is downloaded from release assets, renamed from rgx-cli.rb
    to rgx.rb (class RgxCli -> Rgx) for `brew install brevity1swos/tap/rgx`


## [0.1.8] - 2026-02-22

### Bug Fixes

- Use absolute URL for demo GIF so it renders on crates.io
crates.io doesn't serve repository assets, so relative paths like
  assets/demo.gif don't work. Use the raw.githubusercontent.com URL.

### Features

- Add social preview image (1280x640)
Catppuccin Mocha themed preview showing the TUI with pattern input,
  colored capture group highlights, match results, and explanation panel.
  Includes the generation script for reproducibility.


## [0.1.7] - 2026-02-22

### Bug Fixes

- Add allow-dirty for cargo-dist CI workflow validation
cargo-dist validates that .github/workflows/release.yml matches its
  expected content, but we use a custom dist.yml workflow that integrates
  with release-plz. The allow-dirty = ["ci"] setting skips this check.


## [0.1.6] - 2026-02-22

### Miscellaneous

- Set up cargo-dist v0.30.4 for prebuilt binary distribution
Adds dist.yml workflow triggered by version tags to build binaries for
  5 targets (linux x86_64/aarch64, macOS x86_64/aarch64, Windows x86_64)
  and upload them to GitHub Releases created by release-plz.


## [0.1.5] - 2026-02-22

### Bug Fixes

- Bust GitHub CDN cache for demo GIF
Add query parameter to demo.gif URL to force GitHub's camo CDN
  to fetch the updated image instead of serving the stale cache.


## [0.1.4] - 2026-02-22

### Bug Fixes

- Regenerate demo GIF with working rgx binary
Previous demo GIF was recorded before rgx was installed, showing
  a blank terminal. Regenerated with VHS using bash shell that
  inherits PATH with ~/.cargo/bin.


## [0.1.3] - 2026-02-22

### Bug Fixes

- Update crossterm to 0.29, clean up dead_code allows, add logo
- Bump crossterm from 0.28 to 0.29 to align with ratatui 0.30
  - Remove #![allow(dead_code)] from main.rs, lib.rs, and settings.rs
  - Have main.rs use the rgx library crate instead of re-declaring modules
  - Fix duplicate changelog header
  - Add SVG logo asset
  - Add PCRE2 to engine benchmarks (behind feature gate)


## [0.1.2] - 2026-02-22

### Documentation

- Show demo GIF in README and fix crates.io badge links
Uncomment demo GIF reference and update badge URLs to point to
  rgx-cli on crates.io.

### Features

- Add demo GIF and update dependencies
Generate demo GIF using VHS showing real-time matching, engine
  switching, and flag toggles. Update Cargo.lock after dependency
  bumps from merged dependabot PRs.


## [0.1.1] - 2026-02-22

### Features

- Initial release of rgx — regex101 for the terminal
Interactive TUI with real-time matching, 3 regex engines (Rust regex,
  fancy-regex, PCRE2), capture group highlighting with distinct colors,
  plain-English explanation engine, flag toggles, stdin pipe support,
  and cross-platform support.

  Includes full CI/CD automation (test matrix, clippy, fmt, coverage,
  release-plz, cargo-dist), dependabot config, and issue templates.
