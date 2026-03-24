# Changelog

All notable changes to this project will be documented in this file.

## [0.8.1] - 2026-03-24

### Documentation

- Update README, demo, and r/commandline post for v0.8.0
- README: add --json, --color, --completions, Ctrl+U, -w workspace,
    Ctrl+B benchmark to features, usage, shortcuts, and comparison table.
    Update PCRE2 install instructions (now opt-in).
  - demo.tape: add batch mode section (--json, --color) and Ctrl+U
    regex101 export to interactive section.
  - r_commandline.md: v3 draft with v0.8.0 features for repost.
- Regenerate demo GIF with v0.8.0 features
Includes --json output, --color always, and Ctrl+U regex101 export.
- Bust GIF cache (v=4)


## [0.8.0] - 2026-03-24

### Bug Fixes

- Filter key events to Press-only to prevent WSL double input
On Windows/WSL, crossterm emits Press, Release, and Repeat key events.
  Without filtering, each keystroke produced duplicate characters.
- Remove pcre2-engine from default features
Pre-built binaries dynamically linked to libpcre2, requiring Homebrew on
  macOS. PCRE2 is now opt-in via --all-features or --features pcre2-engine.
  Also adds clap_complete and serde_json dependencies for new features.

### Features

- Add --workspace flag for project-local workspace files
- Add --completions, --json, and --color flags
- --completions <SHELL>: generate shell completions for bash/zsh/fish
    using clap_complete (closes #36)
  - --json: output matches as structured JSON in batch mode (closes #37)
  - --color auto|always|never: ANSI-highlighted match output in batch
    mode, similar to grep --color (closes #38)
- Add regex101 URL export (Ctrl+U) and colored/JSON output support
- Ctrl+U generates a regex101.com URL from current state and copies to
    clipboard, with engine-appropriate flavor mapping (closes #40)
  - print_output() gains color support for highlighted match context
  - print_json_output() produces structured JSON with match positions
    and capture groups


## [0.7.0] - 2026-03-12

### Bug Fixes

- Move --count into print_output, add conflicts_with, update docs
- Move count logic into App::print_output() alongside other output modes
  - Add conflicts_with between --count and --group flags
  - Update README with --count and --group usage examples
  - Update launch playbook with current status and r/commandline v2 draft

### Documentation

- Add vim mode to README and keyboard shortcuts
- Add vim mode to features list and comparison table
  - Add --vim usage example and vim keybinding reference table
  - Add vim_mode to config example
  - Update Esc description to note vim behavior

### Features

- Add Shift+Tab backwards panel cycling and rounded borders option
- Add Shift+Tab (BackTab) to cycle focus backwards through panels
  - Add --rounded CLI flag and rounded_borders config option for rounded
    border characters on all panels and overlays
  - Pass BorderType through all widget structs and overlay functions
- *(vim)* Add Action variants for vim motions and mode transitions
- *(vim)* Add Editor primitives (x, dd, cc, o, O, ^, gg, G, e, paste)
- *(vim)* Create VimState state machine with pending-key dd/cc/gg support
- *(vim)* Add --vim CLI flag, config setting, and App integration
- *(vim)* Wire vim dispatch into event loop with all action handlers
- *(vim)* Show NORMAL/INSERT mode indicator in status bar and update help

### Refactoring

- *(vim)* Simplify dispatch, fix bugs, improve code quality
- Move edit_focused/move_focused to App methods with impl FnOnce,
    eliminating local closures and enabling closure-based dispatch for
    InsertChar and PasteClipboard (removes ~60 lines of boilerplate)
  - Fix EnterNormalMode crossing newline boundaries (add move_left_in_line)
  - Fix o/O reverting to Normal mode when on non-multiline panels
  - Replace stringly-typed vim mode in StatusBar with Option<VimMode> enum
  - Switch undo/redo stacks from Vec to VecDeque for O(1) cap eviction
  - Remove dead MoveToContentEnd and duplicate MoveToLineStart actions
  - Delegate delete_char_at_cursor to delete_forward (identical logic)
  - Add VimState::cancel_insert() to encapsulate mode revert

### Testing

- *(vim)* Add integration tests for vim mode


## [0.6.1] - 2026-03-07

### Refactoring

- Extract print_output method and add CLI flag conflict
- Extract duplicated output block into App::print_output()
  - Add conflicts_with = "print" to --output-pattern flag
  - Remove unnecessary .to_string() clones in batch mode checks
  - Update terminal_trove.md categories and license


## [0.6.0] - 2026-03-06

### Features

- Add non-interactive batch mode and pipeline integration
Add --print/-p flag for non-interactive batch mode that skips the TUI
  entirely when pattern and input are provided. Add --output-pattern/-P
  to capture the final pattern after an interactive session.

  Exit codes: 0 = match found, 1 = no match, 2 = error.
  Input priority: --text > --file > stdin (prevents blocking).

  Update launch posts and playbook with pipeline examples.


## [0.5.2] - 2026-03-02

### Documentation

- Rewrite positioning with honest audience framing
Drop "regex101, but in your terminal" tagline in favor of grounded
  positioning that acknowledges regex101.com as the more capable tool
  overall. Add "Who is this for?" section to README targeting the actual
  niche: remote/SSH work, shell pipelines, and engine-specific testing.

  Split comparison table into terminal alternatives (factual) and vs.
  regex101 (honest prose). Update all launch posts, CLI about string,
  and Cargo.toml description to match.


## [0.5.1] - 2026-02-26

### Documentation

- Revise Show HN post for launch
Tighten copy for HN audience: shorter title, personal pain point
  opening, fewer feature bullets, remove self-promotional comparison
  table, reframe closing CTA around user workflows.
- Revise r/rust post for launch
Tighten for r/rust audience: rename technical section to highlight
  architecture discussion, emphasize trait design and pure Rust build,
  trim feature list, add concrete details that invite technical feedback.


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
