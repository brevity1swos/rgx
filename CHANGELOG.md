# Changelog

All notable changes to this project will be documented in this file.

## [0.10.2] - 2026-04-09

### Bug Fixes

- Pcre2 zero-length match offset bug, replace bare unwrap with expect
- Fix `offset += abs_end + 1` → `offset = abs_end + 1` in PCRE2
    find_matches() — the += caused skipped matches when a zero-length
    match occurred at a non-zero position after the first iteration
  - Replace bare .unwrap() with .expect() on capture group 0 across all
    three engine implementations (rust_regex, fancy, pcre2) and in
    expand_replacement() for peeked iterator values
  - Deduplicate whitespace visualization flush pattern in test_input.rs

### Documentation

- Update CONTRIBUTING.md architecture section
Add codegen, recipe, ansi, workspace, debugger, syntax highlighting,
  and vim mode to the architecture overview. Reflects current v0.10.1
  source tree.


## [0.10.1] - 2026-04-08

### Bug Fixes

- Use runtime PCRE2 version detection instead of hard-coded pcre2-sys constants
  `pcre2::version()` returns compile-time constants baked into `pcre2-sys`,
  not the actual linked library version. This caused the CVE-2025-58050
  guard and status bar warning to trigger incorrectly on systems linking
  against PCRE2 >= 10.46 (e.g. NixOS). Now calls `pcre2_config_8` directly
  to query the real runtime version.

## [0.10.0] - 2026-04-05

### Bug Fixes

- Escape [[tests]] in doc comment to fix rustdoc link error
- Preserve debug cache on miss, extract overlay size constants
- Don't consume debug_cache with .take() when pattern doesn't match;
    use ref check first so cache survives for future reopens
  - Extract OVERLAY_WIDTH/OVERLAY_HEIGHT constants from magic numbers

### Documentation

- Update README with code generation, auto engine, test suite mode
Add new features to feature list, keyboard shortcuts table, usage
  examples, and comparison tables. Add test suite TOML format section.
  Add AUR installation method. Update regex101 comparison to reflect
  that code generation is no longer a gap.
- *(vscode)* Update extension to v0.3.0 with new features
Add code generation, auto engine selection, test suite mode,
  alternating match colors, recipe library, and regex101 export
  to the feature list. Add key shortcuts table. Add AUR install
  method. Bump to v0.3.0.
- Update demo tape with code generation, auto engine, alternating colors
Add Ctrl+G code generation overlay, auto engine selection with
  lookahead pattern, and alternating match colors to the VHS demo.
  Reorganized flow for better pacing.
- Regenerate demo GIF with code generation and auto engine features
Updated demo shows: code generation overlay (Ctrl+G), alternating
  match colors, auto engine selection with lookahead pattern, and
  existing features. Cache-bust v=5.
- Slow down code generation section in demo GIF
Increase sleep durations in the Ctrl+G code generation overlay
  section for better readability. 400ms → 1s per language browse,
  1.5s → 3s for overlay display.
- Update roadmap — move shipped features, add new goals
Code generation, test suite mode, alternating colors, and auto
  engine selection are all shipped. New goals: step-through debugger,
  theme customization, regex101 URL import.
- Add step-through debugger design spec
Design for a PCRE2 callout-based step-through regex debugger with
  dual-cursor visualization, backtrack markers, and heatmap mode.
- Add step-through debugger implementation plan
11 tasks covering FFI layer, data model, offset mapping, UI overlay,
  event loop integration, config, tests, and verification.
- Update README, roadmap, demo, and VS Code extension for debugger
- Add step-through debugger to features, keyboard shortcuts, and
    comparison tables in README
  - Update "vs regex101" section — rgx now has its own debugger
  - Move debugger to "Recently Shipped" in roadmap
  - Add debugger section to demo tape (step-through, heatmap)
  - Bump VS Code extension to v0.4.0 with debugger keywords
  - Fix clippy field_reassign_with_default in debugger_tests.rs
  - Update rust-cache to v2.9.1 (fixes Node.js 20 deprecation)

### Features

- *(vscode)* Improve marketplace discoverability
Add extension icon, richer description, better categories (Debuggers,
  Testing), expanded keywords, gallery banner, and LICENSE file. Bump to
  v0.2.0.
- Alternate highlight colors between adjacent matches
Even/odd matches now use distinct background colors for visual
  distinction, especially when matches are adjacent or dense.
  Applies to both the test string panel and the match list panel.
- Add code generation (Ctrl+G) for 8 languages
Generate ready-to-use code from the current pattern and flags.
  Select a language from the overlay, copies to clipboard.
- Auto-select engine based on pattern features
Detect lookahead, lookbehind, backreferences, recursion, and
  backtracking verbs in the pattern and auto-upgrade to the
  simplest engine that supports them. Never auto-downgrades.

  Shows a status message when auto-switching occurs.
  Includes 14 unit tests for pattern detection.
- Add test suite mode (--test) for CI-integrated regex validation
Run `rgx --test file.toml` to validate regex patterns against
  should-match/should-not-match assertions. Supports multiple files.
  Exit code 0 = all pass, 1 = failures, 2 = error.

  Extends workspace TOML format with optional [[tests]] sections.
  Colored pass/fail output in terminals.
- *(vscode)* Update extension icon with /rgx/ regex delimiter design
- *(debug)* Add data model and offset map builder for step-through debugger
Introduces DebugStep, PatternToken, and DebugTrace structs plus build_offset_map()
  and find_token_at_offset() helpers that walk the regex-syntax AST to map pattern
  byte offsets to human-readable token descriptions.
- *(debug)* Implement PCRE2 callout-based debug_match via raw FFI
Add the core debug_match function that compiles patterns with
  PCRE2_AUTO_CALLOUT, installs a callout handler to collect execution
  steps (including backtrack detection via PCRE2_CALLOUT_BACKTRACK),
  and returns a DebugTrace with steps, heatmap, and match attempt count.

  Manually declares the Pcre2CalloutBlock struct and pcre2_set_callout_8
  function that are blocklisted by pcre2-sys. Adds pcre2-sys as a direct
  dependency behind the pcre2-engine feature flag.
- *(debug)* Add ToggleDebugger action, app state, and config
Wire up Ctrl+D → ToggleDebugger across the action enum, vim global
  shortcuts, App struct (show_debugger, debug_trace, debug_step,
  debug_show_heatmap, debug_error fields + all navigation methods), and
  Settings.debug_max_steps (default 10_000).
- *(debug)* Add debugger overlay UI, event loop handler, and help entry
- Create src/ui/debugger.rs: full-screen RED-bordered overlay with pattern
    panel (YELLOW token highlight), subject panel (TEAL position highlight),
    step/backtrack info, optional heatmap (BLUE→PEACH→RED gradient), capture
    display, and controls footer; gated on pcre2-engine feature
  - Modify src/ui/mod.rs: declare debugger module, inject render_debugger call
    after codegen overlay, add Ctrl+D shortcut to help page 0
  - Modify src/main.rs: add debugger overlay key handler (←/→/h/l/Home/End/g/G/
    m/f/H/Esc/q); fix ToggleDebugger to only open (start_debug) or only close

### Refactoring

- Simplify workspace, codegen, and fix doc link
- Extract engine_kind() and flags() helpers in Workspace to
    deduplicate engine parsing and flag construction
  - Make TestResult.passed() a method instead of stored field
  - Change Language::all() from Vec allocation to static slice
  - Escape [[tests]] in doc comment to fix rustdoc link error
- Extract escape helper and reuse inline_prefix in codegen
- Extract escape_double_quoted() to deduplicate pattern escaping
    in Rust, Python, and Java generators
  - Reuse EngineFlags::to_inline_prefix() for PHP flag building
- Extract collect_flags helper, fix formatting in theme
- Add collect_flags() to deduplicate flag-building in Python, Java,
    and C# code generators
  - Fix missing blank line after match_bg() in theme.rs
- Fix debugger tech debt — dedup overlay, surface errors, tidy imports
- Share centered_overlay() from ui/mod.rs instead of duplicating in debugger.rs
  - Surface debug errors via status bar instead of silent debug_error field
  - Remove unused debug_error field from App
  - Move regex-syntax imports to file top in pcre2_debug.rs
  - Restore launch/r_commandline.md to clean state
- Simplify debugger code after review
- Remove redundant DebugStep::index field (always equals Vec position)
  - Extract panel_block() helper to deduplicate 4 identical Block builders
  - Replace hand-written token descriptions with explain::formatter functions
  - Replace magic callout return values with named constants
  - Collapse find_token_at_offset double-scan into single pass
  - Move centered_overlay import to top of debugger.rs
  - Remove WHAT comments, improve WHY comments
  - Remove section banner comments
  - Remove duplicate integration tests (covered by unit tests)
- Introduce DebugSession, shared parse_ast, char span helper, trace cache, byte_to_token precomputation
Co-locates debugger state into DebugSession struct, extracts shared
  parse_ast helper to avoid duplicated AST parsing, adds build_char_spans
  to DRY the char-iteration pattern, caches debug traces across
  close/reopen, and pre-computes byte_to_token map for O(1) heatmap
  lookups.
- Remove redundant params, use byte_to_token in captures, fix cfg gates
- render_debugger reads pattern/subject from DebugSession instead of
    extra params (8 -> 4 args)
  - render_captures uses byte_to_token O(1) lookup instead of
    find_token_at_offset O(n) scan
  - Remove Option<()> dummy field for non-PCRE2 builds; gate call sites
    with #[cfg] instead
  - close_debug() saves session to cache preserving step/heatmap state
    on reopen
  - Cache stores full DebugSession instead of just the trace
- Resolve pre-existing tech debt
- Extract shared ansi module to deduplicate ANSI escape constants
    between app.rs and workspace.rs
  - Remove dead ThemeSettings struct and catppuccin field (never read)
  - Surface create_dir_all error in workspace save instead of swallowing
  - Hoist inline `use Workspace` imports to top-level in main.rs
  - Move url_encode from nested function to module-level
  - Replace magic numbers with named constants (MAX_PATTERN_HISTORY,
    STATUS_DISPLAY_TICKS, MAX_UNDO_STACK)
  - Document switch_engine_to low-level contract

### Testing

- Add integration tests for step-through debugger
Covers debug_match end-to-end, backtracking detection with heatmap
  validation, offset map accuracy, find_token_at_offset, flag handling
  (case-insensitive, unicode), all gated behind #![cfg(feature = "pcre2-engine")].

### Ci

- Update VS Code extension workflow to Node.js 22
Node.js 20 is deprecated on GitHub Actions runners starting
  June 2, 2026. Upgrade proactively to avoid forced migration.


## [0.9.0] - 2026-04-01

### Bug Fixes

- Block CVE-2025-58050 (*scs:) verb on PCRE2 10.45, add status bar warning
On PCRE2 10.45, patterns invoking the scan-substring verb (*scs:) /
  (*SCAN_SUBSTRING:) are rejected before reaching the vulnerable pcre2_match
  code path. All other PCRE2 patterns are unaffected.

  A persistent red warning badge is shown in the status bar whenever the PCRE2
  engine is active on an affected version, prompting users to upgrade to >= 10.46.

  Documents CVE-2025-58050 in the README PCRE2 install section.

### Documentation

- Add roadmap with next planned features

### Features

- Add 8 text-processing regex recipes
Add real-world patterns inspired by mise CLI tools: VTT/SRT timestamps,
  HTML tags, sentence boundaries, YouTube IDs, IATA codes, Unicode
  combining marks, emoji, and Markdown headings.


## [0.8.1] - 2026-03-25

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
- Update Show HN draft for v0.8.0
- Add editor plugins design spec
- Add editor plugins implementation plan
- Add Editor & Terminal Integration section to README
VS Code, Neovim, Zed, and tmux integration instructions.

### Features

- *(zed)* Add task definitions for rgx integration
- *(vscode)* Add VS Code extension with 3 terminal commands
- rgx: Open — launch rgx in integrated terminal
  - rgx: Open with Selection — pass selected text as --text
  - rgx: Open with Pattern — pass selected text as pattern arg
  - Configurable binary path and default engine

### Miscellaneous

- *(vscode)* Add .gitignore, untrack node_modules/out/.vsix
- Add .env.local to .gitignore

### Refactoring

- Extract clipboard helper, derive Serialize, ANSI constants
- Extract copy_to_clipboard() helper to deduplicate clipboard error
    handling between copy_selected_match() and copy_regex101_url()
  - Derive Serialize on Match/CaptureGroup with serde rename attrs,
    replacing 30 lines of manual JSON construction in print_json_output()
  - Extract ANSI escape codes into named constants (ANSI_RED_BOLD,
    ANSI_GREEN_BOLD, ANSI_RESET)

### Ci

- Add VS Code extension publish workflow
- Restrict dist.yml to v-prefixed semver tags only


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
