# `rgx filter` Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `rgx filter` subcommand that reads stdin or a file line-by-line, live-filters lines through a regex in a TUI, and emits the filtered output to stdout on accept. Ships as either a v0.11.x round-out or v0.12.0 (orthogonal to Road A's core maintenance commitment).

**Architecture:** A new subcommand (`rgx filter …`) with its own CLI surface and `filter` module. Non-interactive paths (`--count`, `--line-number`, non-TTY stdout) bypass the TUI entirely. TUI mode reuses the existing `Editor` widget for the pattern pane, the `RegexEngine` trait for matching, `syntax_highlight::highlight` for pattern colorization, and the existing theme. Filter state (`FilterApp`) is separate from the main `App` — the two modes never share state because their UX is fundamentally different (stream filter vs. fixed test string).

**Tech Stack:** Rust 1.74 MSRV, `clap` (subcommand derive), `ratatui` + `crossterm` for the TUI, `tokio` for the async event loop, `is-terminal` for TTY detection. Existing rgx modules reused as-is: `engine::create_engine`, `engine::EngineFlags`, `input::editor::Editor`, `ui::syntax_highlight::highlight`, `ui::theme`.

**Related:**
- Design rationale: conversation on 2026-04-18 covering synergy with agx (AI trace debugger) and sift (AI write ledger)
- Roadmap item: "interactive grep" bullet on `docs/ROADMAP.md`

---

## Scope check

Single-subsystem plan, no decomposition needed. One new subcommand (`filter`), one new module pair (`filter/mod.rs` + `filter/ui.rs`), plus CLI glue in `config/cli.rs`, entry dispatch in `main.rs`, and documentation.

## File structure

**New files:**
- `src/filter/mod.rs` — `FilterOptions`, input reading (`read_input`), pure filter function (`filter_lines`), exit-code helpers, non-interactive emit path. No async, no TUI dependencies.
- `src/filter/app.rs` — `FilterApp` state struct + pattern recomputation. Mirrors the role of `app.rs` but specific to filter mode. Depends on `grex`-free engine trait layer only.
- `src/filter/ui.rs` — `render(frame, app)` function. Pattern pane + match list + status bar. Depends on ratatui + existing theme.
- `src/filter/run.rs` — TUI entry point (`run_tui(app) -> Result<FilterOutcome>`). Owns the event loop, calls into `ui::render` and `FilterApp` mutations.
- `tests/filter_tests.rs` — unit tests for pure functions, TUI render smoke tests, and CLI integration tests via `assert_cmd` or direct binary invocation.

**Modified files:**
- `Cargo.toml` — add `is-terminal = "0.4"` to `[dependencies]` (preferred over std's unstable `is_terminal`).
- `src/lib.rs` — `pub mod filter;`.
- `src/config/cli.rs` — add `Filter(FilterArgs)` subcommand variant with flags.
- `src/main.rs` — dispatch on `Command::Filter(args)` to `filter::entry(args)` instead of the main TUI.
- `README.md` — add filter to feature list, keyboard shortcuts table, usage examples, and a "Piping synergy" section with agx/sift recipes.
- `docs/ROADMAP.md` — move "interactive grep" from roadmap to shipped (if appropriate for v0.11.x).
- Parent-directory `CLAUDE.md` (`/Users/seongyongpark/project/brevity1swos/CLAUDE.md`) — add `src/filter/` to the architecture file tree.

---

## Task 1: Add `Filter` subcommand to clap CLI

**Files:**
- Modify: `src/config/cli.rs`
- Test: `tests/filter_tests.rs` (new file)

- [ ] **Step 1: Read `src/config/cli.rs` to see the current clap shape**

Run: `grep -n "#\[derive\|Parser\|Subcommand\|Command\|pub struct" src/config/cli.rs | head -20`

Confirm whether the CLI uses a single `Cli` struct (no subcommands) or already has a `Subcommand` enum. Based on the current main-flow CLI, it likely has a single `Cli` struct — this task adds the first subcommand.

- [ ] **Step 2: Add the subcommand enum and `FilterArgs` struct**

Add at the top of `src/config/cli.rs` (after existing imports):

```rust
use clap::{Args, Subcommand};
```

Append (or adjust alongside the existing `Cli` struct — if `Cli` currently has inline args, keep those as the default behavior when no subcommand is given):

```rust
#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Live-filter stdin or a file through a regex (grep-like, with a TUI).
    Filter(FilterArgs),
}

#[derive(Args, Debug, Clone)]
pub struct FilterArgs {
    /// Regex pattern to filter with. If omitted, the TUI starts with an empty pattern.
    #[arg(value_name = "PATTERN")]
    pub pattern: Option<String>,

    /// Read input from a file instead of stdin.
    #[arg(short = 'f', long)]
    pub file: Option<std::path::PathBuf>,

    /// Invert the match — emit lines that do NOT match (like `grep -v`).
    #[arg(short = 'v', long)]
    pub invert: bool,

    /// Emit only the count of matching lines (non-interactive).
    #[arg(short = 'c', long)]
    pub count: bool,

    /// Prefix each emitted line with its line number (non-interactive).
    #[arg(short = 'n', long = "line-number")]
    pub line_number: bool,

    /// Case-insensitive matching (equivalent to Alt+i inside the TUI).
    #[arg(short = 'i', long)]
    pub case_insensitive: bool,
}
```

Then add a top-level `command` field to the existing `Cli` struct (keep it optional so running bare `rgx` still enters the main TUI):

```rust
#[command(subcommand)]
pub command: Option<Command>,
```

- [ ] **Step 3: Write a failing CLI parse test**

Create `tests/filter_tests.rs`:
```rust
use clap::Parser;
use rgx::config::cli::{Cli, Command};

#[test]
fn filter_subcommand_with_pattern_parses() {
    let cli = Cli::try_parse_from(["rgx", "filter", "error"]).unwrap();
    match cli.command {
        Some(Command::Filter(args)) => {
            assert_eq!(args.pattern.as_deref(), Some("error"));
            assert!(!args.invert);
            assert!(!args.count);
            assert!(!args.line_number);
        }
        _ => panic!("expected Filter subcommand"),
    }
}

#[test]
fn filter_subcommand_with_flags_parses() {
    let cli = Cli::try_parse_from(["rgx", "filter", "-vc", "-n", "-f", "log.txt", "error"]).unwrap();
    match cli.command {
        Some(Command::Filter(args)) => {
            assert!(args.invert);
            assert!(args.count);
            assert!(args.line_number);
            assert_eq!(args.file.as_deref().and_then(|p| p.to_str()), Some("log.txt"));
            assert_eq!(args.pattern.as_deref(), Some("error"));
        }
        _ => panic!("expected Filter subcommand"),
    }
}

#[test]
fn bare_rgx_has_no_subcommand() {
    let cli = Cli::try_parse_from(["rgx"]).unwrap();
    assert!(cli.command.is_none());
}
```

- [ ] **Step 4: Run the tests**

Run: `cargo test --test filter_tests`
Expected: PASS for all three. If `Cli` was not previously re-exported publicly, add `pub use config::cli::Cli;` to `src/lib.rs` or reach it via `rgx::config::cli::Cli`.

- [ ] **Step 5: Run clippy + full suite**

Run: `cargo clippy --all-features --all-targets -- -D warnings && cargo test --all-features`
Expected: no warnings, everything passes.

- [ ] **Step 6: Commit**

```bash
git add src/config/cli.rs src/lib.rs tests/filter_tests.rs
git commit -m "feat(filter): add rgx filter subcommand to CLI"
```

---

## Task 2: Create the filter module skeleton + pure `filter_lines`

**Files:**
- Create: `src/filter/mod.rs`
- Modify: `src/lib.rs`
- Test: `tests/filter_tests.rs`

- [ ] **Step 1: Create `src/filter/mod.rs`**

```rust
//! `rgx filter` subcommand — live/non-interactive regex filter over stdin or a file.

use crate::engine::{self, EngineFlags, EngineKind};

#[derive(Debug, Clone, Copy)]
pub struct FilterOptions {
    pub invert: bool,
    pub case_insensitive: bool,
}

impl Default for FilterOptions {
    fn default() -> Self {
        Self { invert: false, case_insensitive: false }
    }
}

impl FilterOptions {
    fn flags(&self) -> EngineFlags {
        let mut flags = EngineFlags::default();
        if self.case_insensitive {
            flags.case_insensitive = true;
        }
        flags
    }
}

/// Apply the pattern to each line. Returns the 0-indexed line numbers of every
/// line whose match status (matches vs. invert) satisfies `options.invert`.
///
/// Returns `Err` if the pattern fails to compile. An empty pattern is treated
/// as "match everything" (every line passes) so the TUI has a sensible default
/// before the user types.
pub fn filter_lines(
    lines: &[String],
    pattern: &str,
    options: FilterOptions,
) -> Result<Vec<usize>, String> {
    if pattern.is_empty() {
        // Empty pattern — every line passes iff not inverted.
        return Ok(if options.invert {
            Vec::new()
        } else {
            (0..lines.len()).collect()
        });
    }

    let engine = engine::create_engine(EngineKind::RustRegex);
    let compiled = engine
        .compile(pattern, &options.flags())
        .map_err(|e| e.to_string())?;

    let mut indices = Vec::with_capacity(lines.len());
    for (idx, line) in lines.iter().enumerate() {
        let matched = !compiled.find_matches(line).is_empty();
        if matched != options.invert {
            indices.push(idx);
        }
    }
    Ok(indices)
}
```

- [ ] **Step 2: Declare the module in `src/lib.rs`**

Add after `pub mod explain;`:
```rust
pub mod filter;
```

- [ ] **Step 3: Add failing unit tests for `filter_lines`**

Append to `tests/filter_tests.rs`:
```rust
use rgx::filter::{filter_lines, FilterOptions};

fn to_lines(strs: &[&str]) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

#[test]
fn empty_pattern_passes_every_line() {
    let lines = to_lines(&["foo", "bar", "baz"]);
    let got = filter_lines(&lines, "", FilterOptions::default()).unwrap();
    assert_eq!(got, vec![0, 1, 2]);
}

#[test]
fn empty_pattern_with_invert_passes_nothing() {
    let lines = to_lines(&["foo", "bar", "baz"]);
    let got = filter_lines(
        &lines,
        "",
        FilterOptions { invert: true, case_insensitive: false },
    )
    .unwrap();
    assert!(got.is_empty());
}

#[test]
fn simple_pattern_selects_matching_lines() {
    let lines = to_lines(&["hello 42", "world", "hello 99", "foo"]);
    let got = filter_lines(&lines, r"\d+", FilterOptions::default()).unwrap();
    assert_eq!(got, vec![0, 2]);
}

#[test]
fn invert_flag_selects_non_matching_lines() {
    let lines = to_lines(&["hello 42", "world", "hello 99", "foo"]);
    let got = filter_lines(
        &lines,
        r"\d+",
        FilterOptions { invert: true, case_insensitive: false },
    )
    .unwrap();
    assert_eq!(got, vec![1, 3]);
}

#[test]
fn case_insensitive_flag() {
    let lines = to_lines(&["Error: boom", "OK", "ERROR again"]);
    let got = filter_lines(
        &lines,
        "error",
        FilterOptions { invert: false, case_insensitive: true },
    )
    .unwrap();
    assert_eq!(got, vec![0, 2]);
}

#[test]
fn invalid_pattern_returns_err() {
    let lines = to_lines(&["a"]);
    let got = filter_lines(&lines, "(unclosed", FilterOptions::default());
    assert!(got.is_err());
}
```

- [ ] **Step 4: Run the tests**

Run: `cargo test --test filter_tests`
Expected: PASS. If `engine::create_engine` isn't visible, confirm the module is `pub mod engine;` in `lib.rs` (it is). If `CompiledRegex::find_matches` has a different name in the existing trait, adjust — typical alternates are `matches` or `find_iter`.

Grep to verify: `grep -n "fn find_matches\|fn matches\|pub trait CompiledRegex" src/engine/mod.rs`

- [ ] **Step 5: Clippy + full suite**

Run: `cargo clippy --all-features --all-targets -- -D warnings && cargo test --all-features`
Expected: all pass.

- [ ] **Step 6: Commit**

```bash
git add src/filter/mod.rs src/lib.rs tests/filter_tests.rs
git commit -m "feat(filter): add filter_lines pure function with tests"
```

---

## Task 3: Input reader (stdin or file)

**Files:**
- Modify: `src/filter/mod.rs`
- Test: `tests/filter_tests.rs`

- [ ] **Step 1: Add `read_input` helper**

Append to `src/filter/mod.rs`:
```rust
use std::io::{self, BufRead, BufReader, Read};
use std::path::Path;

/// Read all lines from either a file path or the provided reader (typically stdin).
/// Trailing `\n`/`\r\n` is stripped per line. A trailing empty line (from a
/// terminating newline) is dropped.
pub fn read_input(file: Option<&Path>, fallback: impl Read) -> io::Result<Vec<String>> {
    let reader: Box<dyn BufRead> = match file {
        Some(path) => Box::new(BufReader::new(std::fs::File::open(path)?)),
        None => Box::new(BufReader::new(fallback)),
    };
    let mut out = Vec::new();
    for line in reader.lines() {
        out.push(line?);
    }
    Ok(out)
}
```

- [ ] **Step 2: Write failing file-reader test**

Append to `tests/filter_tests.rs`:
```rust
use rgx::filter::read_input;
use std::io::Cursor;

#[test]
fn read_input_from_in_memory_stdin() {
    let data = "foo\nbar\nbaz\n";
    let got = read_input(None, Cursor::new(data)).unwrap();
    assert_eq!(got, vec!["foo", "bar", "baz"]);
}

#[test]
fn read_input_from_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("input.txt");
    std::fs::write(&path, "alpha\nbeta\n").unwrap();
    let got = read_input(Some(&path), Cursor::new("ignored")).unwrap();
    assert_eq!(got, vec!["alpha", "beta"]);
}
```

Note: `tempfile` is already a dev-dependency per `Cargo.toml`. Confirm via `grep tempfile Cargo.toml`.

- [ ] **Step 3: Run tests**

Run: `cargo test --test filter_tests read_input`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add src/filter/mod.rs tests/filter_tests.rs
git commit -m "feat(filter): add read_input helper for stdin or file source"
```

---

## Task 4: Non-interactive output paths (count + emit matching lines)

**Files:**
- Modify: `src/filter/mod.rs`
- Test: `tests/filter_tests.rs`

- [ ] **Step 1: Add non-interactive entry points**

Append to `src/filter/mod.rs`:
```rust
use std::io::Write;

/// Exit codes, matching grep conventions.
pub const EXIT_MATCH: i32 = 0;
pub const EXIT_NO_MATCH: i32 = 1;
pub const EXIT_ERROR: i32 = 2;

/// Emit matching lines to `writer`. If `line_number` is true, each line is
/// prefixed with its 1-indexed line number and a colon.
pub fn emit_matches(
    writer: &mut dyn Write,
    lines: &[String],
    matched: &[usize],
    line_number: bool,
) -> io::Result<()> {
    for &idx in matched {
        if line_number {
            writeln!(writer, "{}:{}", idx + 1, lines[idx])?;
        } else {
            writeln!(writer, "{}", lines[idx])?;
        }
    }
    Ok(())
}

/// Emit only the count of matched lines.
pub fn emit_count(writer: &mut dyn Write, matched_count: usize) -> io::Result<()> {
    writeln!(writer, "{matched_count}")
}
```

- [ ] **Step 2: Write failing tests**

Append to `tests/filter_tests.rs`:
```rust
use rgx::filter::{emit_count, emit_matches};

#[test]
fn emit_matches_plain() {
    let lines = to_lines(&["alpha", "beta", "gamma"]);
    let matched = vec![0, 2];
    let mut buf = Vec::new();
    emit_matches(&mut buf, &lines, &matched, false).unwrap();
    assert_eq!(String::from_utf8(buf).unwrap(), "alpha\ngamma\n");
}

#[test]
fn emit_matches_with_line_numbers() {
    let lines = to_lines(&["alpha", "beta", "gamma"]);
    let matched = vec![0, 2];
    let mut buf = Vec::new();
    emit_matches(&mut buf, &lines, &matched, true).unwrap();
    assert_eq!(String::from_utf8(buf).unwrap(), "1:alpha\n3:gamma\n");
}

#[test]
fn emit_count_writes_number() {
    let mut buf = Vec::new();
    emit_count(&mut buf, 7).unwrap();
    assert_eq!(String::from_utf8(buf).unwrap(), "7\n");
}
```

- [ ] **Step 3: Run**

Run: `cargo test --test filter_tests emit_`
Expected: PASS.

- [ ] **Step 4: Commit**

```bash
git add src/filter/mod.rs tests/filter_tests.rs
git commit -m "feat(filter): emit_matches and emit_count non-interactive helpers"
```

---

## Task 5: TTY-aware dispatcher entry function

**Files:**
- Modify: `Cargo.toml` (add `is-terminal`)
- Modify: `src/filter/mod.rs`
- Test: `tests/filter_tests.rs`

- [ ] **Step 1: Add `is-terminal` dependency**

Edit `Cargo.toml`, add under `[dependencies]`:
```toml
is-terminal = "0.4"
```

Run: `cargo fetch`
Expected: resolves cleanly.

- [ ] **Step 2: Add `entry` dispatcher**

Append to `src/filter/mod.rs`:
```rust
use crate::config::cli::FilterArgs;

/// CLI entry point for `rgx filter`. Reads input, decides between non-interactive
/// and TUI modes, and returns an exit code.
pub fn entry(args: FilterArgs) -> i32 {
    match run_entry(args) {
        Ok(code) => code,
        Err(msg) => {
            eprintln!("rgx filter: {msg}");
            EXIT_ERROR
        }
    }
}

fn run_entry(args: FilterArgs) -> Result<i32, String> {
    let lines = read_input(args.file.as_deref(), io::stdin())
        .map_err(|e| format!("reading input: {e}"))?;

    let options = FilterOptions {
        invert: args.invert,
        case_insensitive: args.case_insensitive,
    };

    // Non-interactive paths: --count, --line-number, or a pattern was given and
    // stdout is not a TTY (so we're being piped).
    let has_pattern = args.pattern.as_deref().is_some_and(|p| !p.is_empty());
    let stdout_is_tty = is_terminal::IsTerminal::is_terminal(&io::stdout());
    let non_interactive = args.count
        || args.line_number
        || (has_pattern && !stdout_is_tty);

    if non_interactive {
        let pattern = args.pattern.unwrap_or_default();
        let matched = filter_lines(&lines, &pattern, options).map_err(|e| format!("pattern: {e}"))?;

        let mut stdout = io::stdout().lock();
        if args.count {
            emit_count(&mut stdout, matched.len())
                .map_err(|e| format!("writing output: {e}"))?;
        } else {
            emit_matches(&mut stdout, &lines, &matched, args.line_number)
                .map_err(|e| format!("writing output: {e}"))?;
        }
        return Ok(if matched.is_empty() { EXIT_NO_MATCH } else { EXIT_MATCH });
    }

    // TUI mode — implemented in Task 8+.
    // Returning a clear stub message for now so developers can track progress.
    Err("TUI mode not yet implemented (see Task 8)".to_string())
}
```

- [ ] **Step 3: Write failing test for non-interactive count path**

Append to `tests/filter_tests.rs`:
```rust
use rgx::config::cli::FilterArgs;
use rgx::filter::entry;

fn filter_args(pattern: Option<&str>) -> FilterArgs {
    FilterArgs {
        pattern: pattern.map(str::to_string),
        file: None,
        invert: false,
        count: false,
        line_number: false,
        case_insensitive: false,
    }
}

// Direct call via entry() doesn't let us inject stdin/stdout easily — so we
// test the composition (filter_lines + emit_count) end-to-end below with a
// file input. Integration tests for the real binary live in Task 6.

#[test]
fn count_mode_returns_expected_count() {
    let lines = to_lines(&["one 1", "two", "three 3", "four 4"]);
    let options = FilterOptions::default();
    let matched = filter_lines(&lines, r"\d", options).unwrap();
    let mut buf = Vec::new();
    emit_count(&mut buf, matched.len()).unwrap();
    assert_eq!(String::from_utf8(buf).unwrap(), "3\n");
}
```

- [ ] **Step 4: Run**

Run: `cargo test --test filter_tests`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml Cargo.lock src/filter/mod.rs tests/filter_tests.rs
git commit -m "feat(filter): TTY-aware entry dispatcher with non-interactive paths"
```

---

## Task 6: Wire `Command::Filter` in main.rs + end-to-end CLI test

**Files:**
- Modify: `src/main.rs`
- Test: `tests/filter_tests.rs`

- [ ] **Step 1: Dispatch Filter command in main.rs**

Locate the top of `fn main()` where CLI parsing happens. Before the existing TUI startup, add:
```rust
if let Some(rgx::config::cli::Command::Filter(args)) = cli.command.clone() {
    std::process::exit(rgx::filter::entry(args));
}
```

Adjust placement so it's the first branch after CLI parse (before any terminal raw-mode setup — the filter's non-interactive paths must not touch the terminal).

- [ ] **Step 2: Build the binary**

Run: `cargo build --all-features`
Expected: compiles.

- [ ] **Step 3: Write an end-to-end test invoking the real binary**

Append to `tests/filter_tests.rs`:
```rust
use std::io::Write as _;
use std::process::{Command, Stdio};

fn rgx_bin() -> std::path::PathBuf {
    // Cargo puts integration test binaries next to the main binary under target/debug.
    let mut p = std::env::current_exe().unwrap();
    p.pop(); // test binary name
    if p.ends_with("deps") {
        p.pop();
    }
    p.push(if cfg!(windows) { "rgx.exe" } else { "rgx" });
    p
}

#[test]
fn cli_filter_count_reads_stdin() {
    let bin = rgx_bin();
    assert!(bin.exists(), "rgx binary not found at {bin:?}; build first");
    let mut child = Command::new(&bin)
        .args(["filter", "--count", r"\d+"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(b"error 1\nok\nerror 2\nwarn\n")
        .unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "2");
}

#[test]
fn cli_filter_emit_matching_lines_from_file() {
    let bin = rgx_bin();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("log.txt");
    std::fs::write(&path, "info: ok\nerror: boom\ninfo: ok2\nerror: kaboom\n").unwrap();
    let out = Command::new(&bin)
        .args(["filter", "-f", path.to_str().unwrap(), "-n", "error"])
        .stderr(Stdio::piped())
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0));
    assert_eq!(
        String::from_utf8_lossy(&out.stdout),
        "2:error: boom\n4:error: kaboom\n"
    );
}

#[test]
fn cli_filter_no_match_returns_exit_1() {
    let bin = rgx_bin();
    let mut child = Command::new(&bin)
        .args(["filter", "--count", "zzz"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.as_mut().unwrap().write_all(b"foo\nbar\n").unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(1));
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "0");
}

#[test]
fn cli_filter_invalid_pattern_returns_exit_2() {
    let bin = rgx_bin();
    let mut child = Command::new(&bin)
        .args(["filter", "--count", "(unclosed"])
        .stdin(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.as_mut().unwrap().write_all(b"foo\n").unwrap();
    let out = child.wait_with_output().unwrap();
    assert_eq!(out.status.code(), Some(2));
}
```

- [ ] **Step 4: Run the CLI tests**

Run: `cargo build && cargo test --test filter_tests`
Expected: all CLI tests PASS.

Note: `cargo test` automatically builds the binary before running tests in the same package, but only if the test depends on it. The `rgx_bin()` helper explicitly asserts the path exists — if it doesn't, run `cargo build` first.

- [ ] **Step 5: Clippy**

Run: `cargo clippy --all-features --all-targets -- -D warnings`
Expected: no warnings.

- [ ] **Step 6: Commit**

```bash
git add src/main.rs tests/filter_tests.rs
git commit -m "feat(filter): wire Command::Filter in main.rs + e2e CLI tests"
```

---

## Task 7: `FilterApp` state struct

**Files:**
- Create: `src/filter/app.rs`
- Modify: `src/filter/mod.rs` (re-export)
- Test: `tests/filter_tests.rs`

- [ ] **Step 1: Create `src/filter/app.rs`**

```rust
//! TUI-mode state for `rgx filter`.

use crate::engine::{self, CompiledRegex, EngineFlags, EngineKind, RegexEngine};
use crate::filter::FilterOptions;
use crate::input::editor::Editor;

pub struct FilterApp {
    pub pattern_editor: Editor,
    pub options: FilterOptions,
    pub lines: Vec<String>,
    /// Indices of `lines` that currently match the pattern.
    pub matched: Vec<usize>,
    /// Selected index into `matched` for the cursor in the match list.
    pub selected: usize,
    /// Scroll offset (first visible index into `matched`).
    pub scroll: usize,
    /// Compilation error from the last `recompute`, if any.
    pub error: Option<String>,
    /// Whether to quit the event loop on next tick.
    pub should_quit: bool,
    /// Outcome decided by the user: emit the filtered output, or discard.
    pub outcome: Outcome,
    engine: Box<dyn RegexEngine>,
    engine_flags: EngineFlags,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Pending,
    Emit,
    Discard,
}

impl FilterApp {
    pub fn new(lines: Vec<String>, initial_pattern: &str, options: FilterOptions) -> Self {
        let pattern_editor = Editor::with_content(initial_pattern.to_string());
        let engine_flags = EngineFlags {
            case_insensitive: options.case_insensitive,
            ..EngineFlags::default()
        };
        let engine = engine::create_engine(EngineKind::RustRegex);
        let mut app = Self {
            pattern_editor,
            options,
            lines,
            matched: Vec::new(),
            selected: 0,
            scroll: 0,
            error: None,
            should_quit: false,
            outcome: Outcome::Pending,
            engine,
            engine_flags,
        };
        app.recompute();
        app
    }

    pub fn pattern(&self) -> &str {
        self.pattern_editor.content()
    }

    pub fn recompute(&mut self) {
        self.error = None;
        let pattern = self.pattern().to_string();
        if pattern.is_empty() {
            self.matched = if self.options.invert {
                Vec::new()
            } else {
                (0..self.lines.len()).collect()
            };
            self.clamp_selection();
            return;
        }
        match self.engine.compile(&pattern, &self.engine_flags) {
            Ok(compiled) => {
                self.matched = self.collect_matches(&*compiled);
                self.clamp_selection();
            }
            Err(err) => {
                self.error = Some(err.to_string());
                self.matched.clear();
                self.selected = 0;
                self.scroll = 0;
            }
        }
    }

    fn collect_matches(&self, compiled: &dyn CompiledRegex) -> Vec<usize> {
        let mut out = Vec::with_capacity(self.lines.len());
        for (idx, line) in self.lines.iter().enumerate() {
            let hit = !compiled.find_matches(line).is_empty();
            if hit != self.options.invert {
                out.push(idx);
            }
        }
        out
    }

    fn clamp_selection(&mut self) {
        if self.matched.is_empty() {
            self.selected = 0;
            self.scroll = 0;
        } else if self.selected >= self.matched.len() {
            self.selected = self.matched.len() - 1;
        }
    }

    pub fn select_next(&mut self) {
        if self.selected + 1 < self.matched.len() {
            self.selected += 1;
        }
    }

    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }

    pub fn toggle_case_insensitive(&mut self) {
        self.options.case_insensitive = !self.options.case_insensitive;
        self.engine_flags.case_insensitive = self.options.case_insensitive;
        self.recompute();
    }

    pub fn toggle_invert(&mut self) {
        self.options.invert = !self.options.invert;
        self.recompute();
    }
}
```

- [ ] **Step 2: Re-export `FilterApp` from the module**

Add to `src/filter/mod.rs`:
```rust
pub mod app;
pub use app::{FilterApp, Outcome};
```

- [ ] **Step 3: Write failing tests for `FilterApp`**

Append to `tests/filter_tests.rs`:
```rust
use rgx::filter::{FilterApp, Outcome};

#[test]
fn filter_app_empty_pattern_shows_all_lines() {
    let lines = to_lines(&["one", "two", "three"]);
    let app = FilterApp::new(lines, "", FilterOptions::default());
    assert_eq!(app.matched, vec![0, 1, 2]);
    assert_eq!(app.outcome, Outcome::Pending);
    assert!(app.error.is_none());
}

#[test]
fn filter_app_applies_initial_pattern() {
    let lines = to_lines(&["error 1", "ok", "error 2"]);
    let app = FilterApp::new(lines, "error", FilterOptions::default());
    assert_eq!(app.matched, vec![0, 2]);
}

#[test]
fn filter_app_invalid_pattern_sets_error() {
    let lines = to_lines(&["a"]);
    let app = FilterApp::new(lines, "(unclosed", FilterOptions::default());
    assert!(app.error.is_some());
    assert!(app.matched.is_empty());
}

#[test]
fn filter_app_toggle_invert_flips_match_set() {
    let lines = to_lines(&["error 1", "ok", "error 2"]);
    let mut app = FilterApp::new(lines, "error", FilterOptions::default());
    assert_eq!(app.matched, vec![0, 2]);
    app.toggle_invert();
    assert_eq!(app.matched, vec![1]);
}

#[test]
fn filter_app_toggle_case_insensitive_recomputes() {
    let lines = to_lines(&["ERROR one", "ok", "error two"]);
    let mut app = FilterApp::new(lines.clone(), "error", FilterOptions::default());
    assert_eq!(app.matched, vec![2]);
    app.toggle_case_insensitive();
    assert_eq!(app.matched, vec![0, 2]);
}

#[test]
fn filter_app_selection_clamps_on_pattern_change() {
    let lines = to_lines(&["a", "b", "c", "d"]);
    let mut app = FilterApp::new(lines, "", FilterOptions::default());
    app.selected = 3;
    // Change pattern — now only one line matches.
    app.pattern_editor = rgx::input::editor::Editor::with_content("a".to_string());
    app.recompute();
    assert_eq!(app.matched, vec![0]);
    assert_eq!(app.selected, 0);
}
```

- [ ] **Step 4: Run**

Run: `cargo test --test filter_tests filter_app_`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/filter/app.rs src/filter/mod.rs tests/filter_tests.rs
git commit -m "feat(filter): FilterApp state struct with recompute logic"
```

---

## Task 8: TUI render function

**Files:**
- Create: `src/filter/ui.rs`
- Modify: `src/filter/mod.rs`
- Test: `tests/filter_tests.rs`

- [ ] **Step 1: Create `src/filter/ui.rs`**

```rust
//! Rendering for `rgx filter` TUI mode.

use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, List, ListItem, Paragraph};
use ratatui::Frame;

use crate::filter::FilterApp;
use crate::ui::theme;

pub fn render(frame: &mut Frame, app: &FilterApp) {
    let area = frame.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(1),
            Constraint::Length(1),
        ])
        .split(area);

    render_pattern_pane(frame, chunks[0], app);
    render_match_list(frame, chunks[1], app);
    render_status(frame, chunks[2], app);
}

fn render_pattern_pane(frame: &mut Frame, area: Rect, app: &FilterApp) {
    let content = app.pattern();
    let style = if app.error.is_some() {
        Style::default().fg(theme::RED)
    } else {
        Style::default().fg(theme::TEXT)
    };
    let title = if app.error.is_some() {
        " Pattern (invalid) "
    } else {
        " Pattern "
    };
    let block = Block::default()
        .title(Span::styled(title, Style::default().fg(theme::BLUE)))
        .borders(Borders::ALL);
    let paragraph = Paragraph::new(Line::from(Span::styled(content.to_string(), style))).block(block);
    frame.render_widget(paragraph, area);
}

fn render_match_list(frame: &mut Frame, area: Rect, app: &FilterApp) {
    if let Some(err) = app.error.as_deref() {
        let block = Block::default().borders(Borders::ALL).title(" Matches ");
        let paragraph = Paragraph::new(Line::from(Span::styled(
            format!("error: {err}"),
            Style::default().fg(theme::RED),
        )))
        .block(block);
        frame.render_widget(paragraph, area);
        return;
    }

    let inner_height = area.height.saturating_sub(2) as usize;
    let start = app.scroll.min(app.matched.len().saturating_sub(1));
    let end = (start + inner_height).min(app.matched.len());
    let visible = &app.matched[start..end];
    let items: Vec<ListItem> = visible
        .iter()
        .enumerate()
        .map(|(visible_idx, &line_idx)| {
            let absolute = start + visible_idx;
            let mut style = Style::default().fg(theme::TEXT);
            if absolute == app.selected {
                style = style.add_modifier(Modifier::REVERSED);
            }
            let content = format!("{:>5}  {}", line_idx + 1, app.lines[line_idx]);
            ListItem::new(Line::from(Span::styled(content, style)))
        })
        .collect();
    let block = Block::default()
        .borders(Borders::ALL)
        .title(Span::styled(
            format!(" Matches ({}/{}) ", app.matched.len(), app.lines.len()),
            Style::default().fg(theme::BLUE),
        ));
    frame.render_widget(List::new(items).block(block), area);
}

fn render_status(frame: &mut Frame, area: Rect, app: &FilterApp) {
    let flags = if app.options.case_insensitive { "i" } else { "-" };
    let invert = if app.options.invert { "v" } else { "-" };
    let text = format!(
        " flags: [{flags}{invert}]   Enter: emit  Esc: discard  Alt+i: case  Alt+v: invert  Up/Down: browse "
    );
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            text,
            Style::default().fg(theme::SUBTEXT),
        ))),
        area,
    );
}
```

- [ ] **Step 2: Declare the module**

Add to `src/filter/mod.rs`:
```rust
pub mod ui;
```

- [ ] **Step 3: Confirm `theme::RED` exists (and grep-verify)**

Run: `grep -n "pub const RED\|pub const TEXT\|pub const BLUE\|pub const SUBTEXT" src/ui/theme.rs`
Expected: all four constants exist. If `RED` is named differently (e.g. `ERROR` or `MAROON`), swap the reference. Looking at existing theme usage, the typical name is `RED` — if the palette uses Catppuccin Mocha, it's `MAROON` or `RED`.

- [ ] **Step 4: Write a failing render smoke test**

Append to `tests/filter_tests.rs`:
```rust
use ratatui::backend::TestBackend;
use ratatui::Terminal;

#[test]
fn filter_ui_render_does_not_panic() {
    let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
    let lines = to_lines(&["alpha", "beta", "gamma"]);
    let app = FilterApp::new(lines, "a", FilterOptions::default());
    terminal
        .draw(|frame| rgx::filter::ui::render(frame, &app))
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let rendered: String = buf.content().iter().map(|c| c.symbol()).collect::<Vec<_>>().join("");
    assert!(rendered.contains("Pattern"));
    assert!(rendered.contains("Matches"));
    assert!(rendered.contains("alpha"));
    assert!(rendered.contains("gamma"));
}

#[test]
fn filter_ui_render_with_invalid_pattern_shows_error() {
    let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
    let lines = to_lines(&["a"]);
    let app = FilterApp::new(lines, "(unclosed", FilterOptions::default());
    terminal
        .draw(|frame| rgx::filter::ui::render(frame, &app))
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let rendered: String = buf.content().iter().map(|c| c.symbol()).collect::<Vec<_>>().join("");
    assert!(rendered.contains("invalid"));
    assert!(rendered.contains("error"));
}
```

- [ ] **Step 5: Run**

Run: `cargo test --test filter_tests filter_ui_`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add src/filter/ui.rs src/filter/mod.rs tests/filter_tests.rs
git commit -m "feat(filter): TUI render function with pattern, match list, status"
```

---

## Task 9: TUI event loop (`run_tui`)

**Files:**
- Create: `src/filter/run.rs`
- Modify: `src/filter/mod.rs`
- Modify: `src/filter/mod.rs` `run_entry` to call `run_tui` when appropriate

- [ ] **Step 1: Create `src/filter/run.rs`**

```rust
//! TUI event loop for `rgx filter`.

use std::io;

use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::execute;
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;

use crate::filter::{FilterApp, Outcome};

/// Run the TUI event loop to completion. Returns when the user hits Enter
/// (emit) or Esc/q (discard). Input events come from crossterm's blocking
/// `event::read()` — the filter UI has no background work, so async is not
/// needed.
pub fn run_tui(mut app: FilterApp) -> io::Result<(FilterApp, Outcome)> {
    let mut stdout = io::stdout();
    enable_raw_mode()?;
    execute!(stdout, EnterAlternateScreen)?;

    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let result = event_loop(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;

    result?;
    let outcome = app.outcome;
    Ok((app, outcome))
}

fn event_loop<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut FilterApp,
) -> io::Result<()> {
    while !app.should_quit {
        terminal.draw(|frame| crate::filter::ui::render(frame, app))?;
        match crossterm::event::read()? {
            Event::Key(key) => handle_key(app, key),
            Event::Resize(_, _) => {}
            _ => {}
        }
    }
    Ok(())
}

fn handle_key(app: &mut FilterApp, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.outcome = Outcome::Discard;
            app.should_quit = true;
        }
        KeyCode::Char('q') if key.modifiers == KeyModifiers::NONE => {
            app.outcome = Outcome::Discard;
            app.should_quit = true;
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.outcome = Outcome::Discard;
            app.should_quit = true;
        }
        KeyCode::Enter => {
            app.outcome = Outcome::Emit;
            app.should_quit = true;
        }
        KeyCode::Up => app.select_prev(),
        KeyCode::Down => app.select_next(),
        KeyCode::Char('i') if key.modifiers.contains(KeyModifiers::ALT) => {
            app.toggle_case_insensitive();
        }
        KeyCode::Char('v') if key.modifiers.contains(KeyModifiers::ALT) => {
            app.toggle_invert();
        }
        KeyCode::Backspace => {
            app.pattern_editor.delete_back();
            app.recompute();
        }
        KeyCode::Left => app.pattern_editor.move_left(),
        KeyCode::Right => app.pattern_editor.move_right(),
        KeyCode::Home => app.pattern_editor.move_home(),
        KeyCode::End => app.pattern_editor.move_end(),
        KeyCode::Char(c) if !key.modifiers.contains(KeyModifiers::CONTROL) => {
            app.pattern_editor.insert_char(c);
            app.recompute();
        }
        _ => {}
    }
}
```

- [ ] **Step 2: Declare the module**

Add to `src/filter/mod.rs`:
```rust
pub mod run;
```

- [ ] **Step 3: Replace the TUI stub in `run_entry`**

In `src/filter/mod.rs`, replace the final block of `run_entry` (the `Err("TUI mode not yet implemented...")` branch) with:

```rust
    // TUI mode.
    let initial_pattern = args.pattern.unwrap_or_default();
    let app = FilterApp::new(lines, &initial_pattern, options);
    let (final_app, outcome) = run::run_tui(app).map_err(|e| format!("tui: {e}"))?;

    match outcome {
        Outcome::Emit => {
            let mut stdout = io::stdout().lock();
            emit_matches(&mut stdout, &final_app.lines, &final_app.matched, false)
                .map_err(|e| format!("writing output: {e}"))?;
            Ok(if final_app.matched.is_empty() { EXIT_NO_MATCH } else { EXIT_MATCH })
        }
        Outcome::Discard => Ok(EXIT_NO_MATCH),
        Outcome::Pending => Ok(EXIT_ERROR),
    }
```

- [ ] **Step 4: Write a failing test for key handling**

Extract the key handler by adding a pub-within-crate test hook. In `src/filter/run.rs`, mark `handle_key` as `pub(crate)`:

```rust
pub(crate) fn handle_key(app: &mut FilterApp, key: KeyEvent) { /* ... */ }
```

Append to `tests/filter_tests.rs`:
```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rgx::filter::run::handle_key;

#[test]
fn handle_key_enter_sets_emit() {
    let lines = to_lines(&["x"]);
    let mut app = FilterApp::new(lines, "x", FilterOptions::default());
    handle_key(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!(app.outcome, Outcome::Emit);
    assert!(app.should_quit);
}

#[test]
fn handle_key_esc_sets_discard() {
    let lines = to_lines(&["x"]);
    let mut app = FilterApp::new(lines, "x", FilterOptions::default());
    handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert_eq!(app.outcome, Outcome::Discard);
    assert!(app.should_quit);
}

#[test]
fn handle_key_alt_v_toggles_invert() {
    let lines = to_lines(&["error", "ok"]);
    let mut app = FilterApp::new(lines, "error", FilterOptions::default());
    assert_eq!(app.matched, vec![0]);
    handle_key(&mut app, KeyEvent::new(KeyCode::Char('v'), KeyModifiers::ALT));
    assert_eq!(app.matched, vec![1]);
}

#[test]
fn handle_key_alt_i_toggles_case() {
    let lines = to_lines(&["ERROR", "ok"]);
    let mut app = FilterApp::new(lines, "error", FilterOptions::default());
    assert!(app.matched.is_empty());
    handle_key(&mut app, KeyEvent::new(KeyCode::Char('i'), KeyModifiers::ALT));
    assert_eq!(app.matched, vec![0]);
}

#[test]
fn handle_key_typing_refilters() {
    let lines = to_lines(&["alpha", "beta", "gamma"]);
    let mut app = FilterApp::new(lines, "", FilterOptions::default());
    assert_eq!(app.matched.len(), 3);
    handle_key(&mut app, KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE));
    // Pattern is now "a" — matches alpha, beta, gamma all contain 'a'.
    assert_eq!(app.matched.len(), 3);
    handle_key(&mut app, KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE));
    // Pattern is "al" — only alpha matches.
    assert_eq!(app.matched, vec![0]);
}

#[test]
fn handle_key_backspace_refilters() {
    let lines = to_lines(&["alpha", "beta", "gamma"]);
    let mut app = FilterApp::new(lines, "al", FilterOptions::default());
    assert_eq!(app.matched, vec![0]);
    handle_key(&mut app, KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE));
    // Back to "a" — all three match.
    assert_eq!(app.matched.len(), 3);
}
```

- [ ] **Step 5: Run**

Run: `cargo test --test filter_tests handle_key_`
Expected: PASS.

- [ ] **Step 6: Clippy + full suite**

Run: `cargo clippy --all-features --all-targets -- -D warnings && cargo test --all-features`
Expected: all pass.

- [ ] **Step 7: Commit**

```bash
git add src/filter/run.rs src/filter/mod.rs tests/filter_tests.rs
git commit -m "feat(filter): TUI event loop with key handling and emit/discard outcomes"
```

---

## Task 10: Manual TUI smoke test + pipe sanity check

**Files:** none (manual verification)

- [ ] **Step 1: Build release**

Run: `cargo build --release --all-features`
Expected: builds clean.

- [ ] **Step 2: Interactive smoke test**

Run: `printf 'error 1\nok\nerror 2\nwarn\n' | ./target/release/rgx filter`
Expected: TUI opens showing all 4 lines. Type "error" → match count drops to 2. Type Alt+v → match count flips to 2 (the others). Alt+v back. Press Enter → TUI closes, stdout has the two matching lines.

- [ ] **Step 3: Piped non-TTY test**

Run: `printf 'error 1\nok\nerror 2\nwarn\n' | ./target/release/rgx filter error | wc -l`
Expected: `2`. Non-interactive path was taken because stdout is piped to `wc`.

- [ ] **Step 4: `--count` test**

Run: `printf 'error 1\nok\nerror 2\nwarn\n' | ./target/release/rgx filter --count error`
Expected: `2` on stdout, exit code 0.

- [ ] **Step 5: `--invert` test**

Run: `printf 'error 1\nok\nerror 2\nwarn\n' | ./target/release/rgx filter -v error`
Expected: `ok\nwarn\n` on stdout.

- [ ] **Step 6: Report results**

Document any deviations in the commit message of the next task. If everything works, proceed to Task 11.

---

## Task 11: README update — feature list, shortcuts, synergy recipes

**Files:**
- Modify: `README.md`

- [ ] **Step 1: Add to the feature list**

Locate the bullet list around line 55-63 (where "Code generation", "Auto engine selection", "Generate regex from examples", etc. live). Add after the grex bullet:

```markdown
- **Live filter mode** — `rgx filter [PATTERN]` reads stdin or a file, shows an interactive TUI where you refine a regex against the stream, and emits matching lines on Enter. Supports `--invert`, `--count`, `--line-number`, `--case-insensitive`, and piping (non-TTY stdout skips the TUI entirely).
```

- [ ] **Step 2: Add to the keyboard shortcuts table for filter mode**

After the existing shortcuts table (around line 203), add a new table:

```markdown
### Filter mode (`rgx filter`) shortcuts

| Key | Action |
|-----|--------|
| `Up/Down` | Browse matching lines |
| `Alt+i` | Toggle case-insensitive |
| `Alt+v` | Toggle invert match |
| `Enter` | Emit matched lines to stdout and exit (exit 0) |
| `Esc` / `q` / `Ctrl+C` | Discard and exit (exit 1) |
| Typing / Backspace | Edit the regex pattern (re-filters live) |
```

- [ ] **Step 3: Add a piping & synergy recipes section**

After the shortcuts tables, add:

```markdown
## Piping recipes

`rgx filter` detects whether stdout is a terminal. If it's piped, the TUI is
skipped — it behaves like a regular `grep`-style filter.

### General

```bash
# Count error lines in a log
cat /var/log/system.log | rgx filter --count 'error|fail'

# Emit only non-matching lines (like grep -v)
cat access.log | rgx filter -v '200 '

# Prefix matches with line numbers
rgx filter -f server.log -n 'Exception'
```

### With [agx](https://github.com/brevity1swos/agx)

agx emits AI agent session timelines as Markdown or JSON. Pipe the export
through `rgx filter` to iterate on a regex over the whole session without
re-running agx:

```bash
# Find all steps where a tool call mentions a file path
agx session.jsonl --export md | rgx filter '\btool_use\b.*\.rs'

# Count how many error responses showed up in a session
agx session.jsonl --export md | rgx filter --count '(?i)error'
```

### With [sift](https://github.com/brevity1swos/sift)

sift shows diffs of AI-generated file writes. Use `rgx filter` to prototype
detection rules against real diffs before encoding them into a sift policy:

```bash
# See every diff line that adds a console.log call
sift diff | rgx filter '^\+.*console\.log'

# Test whether a policy regex catches what you expect
sift diff --session current | rgx filter --count '^\+.*TODO'
```

Once a pattern matches what you want in `rgx filter`, copy-paste it into the
appropriate `.sift/policy.yml` rule.
```

- [ ] **Step 4: Commit**

```bash
git add README.md
git commit -m "docs(filter): README feature list, shortcuts table, and piping recipes"
```

---

## Task 12: Architecture documentation + roadmap update

**Files:**
- Modify: `/Users/seongyongpark/project/brevity1swos/CLAUDE.md` (parent dir — not in the git repo, persists to disk only)
- Modify: `docs/ROADMAP.md`

- [ ] **Step 1: Update parent-dir CLAUDE.md**

In `/Users/seongyongpark/project/brevity1swos/CLAUDE.md`, in the architecture src/ tree, add under `grex_integration.rs`:

```
  filter/
    mod.rs            # `rgx filter` entry point, FilterOptions, filter_lines, read_input, non-interactive emit paths
    app.rs            # FilterApp state (pattern editor, matched lines, selection, outcome)
    ui.rs             # TUI render — pattern pane + match list + status bar
    run.rs            # TUI event loop (handle_key, run_tui)
```

And in the Key Patterns section, add:

```markdown
- **Filter subcommand**: `rgx filter [PATTERN]` opens a separate TUI mode (own `FilterApp`, own event loop in `filter::run::run_tui`) for live-filtering stdin or a file. Reuses the `RegexEngine` trait and `Editor` widget but intentionally does not share state with the main `App` — the two modes have fundamentally different UX (stream filter vs. fixed test string). Non-interactive paths (`--count`, `--line-number`, non-TTY stdout) bypass the TUI entirely for clean piping semantics.
```

- [ ] **Step 2: Update `docs/ROADMAP.md`**

Locate the "v0.11.0 — final polish release" section (or wherever "interactive grep" currently lives). Move the interactive grep bullet from "in scope / round-out" to "shipped" (or to a new v0.11.x / v0.12.0 section as appropriate). Update the phrasing to reference `rgx filter`.

If the roadmap already has a "shipped" subsection for v0.11.x:

```markdown
- **`rgx filter`** — interactive grep mode with stdin/file input, live regex refinement, `--invert`/`--count`/`--line-number` flags, and non-TTY piping. Synergizes with agx and sift pipelines. Shipped 2026-04-18 (v0.11.x or v0.12.0).
```

- [ ] **Step 3: Commit roadmap**

```bash
git add docs/ROADMAP.md
git commit -m "docs(filter): move rgx filter to shipped in roadmap"
```

(CLAUDE.md in the parent directory is not tracked by this repo — save the edit to disk but no commit.)

---

## Task 13: Final verification + push

**Files:** none (verification only)

- [ ] **Step 1: Run full clippy**

Run: `cargo clippy --all-features --all-targets -- -D warnings`
Expected: no warnings.

- [ ] **Step 2: Run full test suite**

Run: `cargo test --all-features`
Expected: all tests pass. Expected new test count: ~20 filter-specific tests on top of the existing 255.

- [ ] **Step 3: Run `cargo fmt --check`**

Run: `cargo fmt --check`
Expected: no diff.

- [ ] **Step 4: Push**

```bash
git push origin main
```

Expected: push succeeds. release-plz will pick up the `feat(filter):` commits on its next run.

---

## Self-review checklist

**Spec coverage (Scope B):**
- Live TUI over stdin/file with pattern refinement: Tasks 7, 8, 9 ✓
- `--invert`: Task 7 (toggle) + Task 5 (CLI flag) + Task 9 (Alt+v) ✓
- `--count`: Task 4 (emit) + Task 5 (dispatch) + Task 6 (CLI test) ✓
- `--line-number`: Task 4 (emit) + Task 5 (dispatch) + Task 6 (CLI test) ✓
- Non-TTY piping: Task 5 (`stdout_is_tty` check) + Task 10 (manual verify) ✓
- Exit codes (0/1/2): Task 4 + Task 5 + Task 6 tests ✓
- README synergy recipes (agx + sift): Task 11 ✓

**Scope C items explicitly out:**
- `--json <path>` JSONL field extraction — deferred. Users can `jq -r '.path'` before piping.
- Multi-file input — deferred. `cat file1 file2 | rgx filter ...` works today.

**Placeholder scan:** every code block contains complete, compilable code. Known fallback points are flagged with grep commands to verify (`find_matches` method name in Task 2 Step 4; `theme::RED` constant in Task 8 Step 3).

**Type consistency:**
- `FilterOptions`: fields `invert`, `case_insensitive` used consistently
- `FilterApp`: fields `pattern_editor`, `options`, `lines`, `matched`, `selected`, `scroll`, `error`, `should_quit`, `outcome` used consistently
- `Outcome`: variants `Pending`, `Emit`, `Discard` used consistently
- `FilterArgs`: CLI field names (`pattern`, `file`, `invert`, `count`, `line_number`, `case_insensitive`) match the clap `#[arg]` attributes

**Known gaps:**
1. The plan assumes `CompiledRegex::find_matches` returns a non-empty slice when there's a match. Confirm at Task 2 Step 4 via grep — adjust method name if the actual trait uses a different signature (e.g. `matches_iter`, `find_iter`). Fallback: add a boolean convenience method on `CompiledRegex` in the engine trait.
2. `EngineFlags::case_insensitive` field access in Task 7 assumes the struct exposes that field publicly. If it doesn't, use the existing toggle method (`EngineFlags::toggle_case_insensitive`) or add a constructor.
3. Very large input files will be loaded entirely into memory (`read_input` collects to `Vec<String>`). An explicit `--max-lines` cap is deferred to a follow-up if real users hit this.

---

## Execution options

Plan complete and saved to `docs/superpowers/plans/2026-04-18-rgx-filter.md`. Two execution options:

**1. Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration.

**2. Inline Execution** — Execute tasks in this session using `executing-plans`, batched execution with checkpoints at every 3-4 tasks for review.

Which approach?
