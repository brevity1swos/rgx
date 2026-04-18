//! `rgx filter` subcommand — live/non-interactive regex filter over stdin or a file.

use std::io::{self, BufRead, BufReader, IsTerminal, Read, Write};
use std::path::Path;

use crate::config::cli::FilterArgs;
use crate::engine::{self, EngineFlags, EngineKind};

pub mod app;
pub mod run;
pub mod ui;
pub use app::{FilterApp, Outcome};

#[derive(Debug, Clone, Copy, Default)]
pub struct FilterOptions {
    pub invert: bool,
    pub case_insensitive: bool,
}

impl FilterOptions {
    fn flags(&self) -> EngineFlags {
        EngineFlags {
            case_insensitive: self.case_insensitive,
            ..EngineFlags::default()
        }
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
        let matched = compiled
            .find_matches(line)
            .map(|v| !v.is_empty())
            .unwrap_or(false);
        if matched != options.invert {
            indices.push(idx);
        }
    }
    Ok(indices)
}

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
    let lines =
        read_input(args.file.as_deref(), io::stdin()).map_err(|e| format!("reading input: {e}"))?;

    let options = FilterOptions {
        invert: args.invert,
        case_insensitive: args.case_insensitive,
    };

    // Non-interactive paths: --count, --line-number, or a pattern was given and
    // stdout is not a TTY (so we're being piped).
    let has_pattern = args.pattern.as_deref().is_some_and(|p| !p.is_empty());
    let stdout_is_tty = io::stdout().is_terminal();
    let non_interactive = args.count || args.line_number || (has_pattern && !stdout_is_tty);

    if non_interactive {
        let pattern = args.pattern.unwrap_or_default();
        let matched =
            filter_lines(&lines, &pattern, options).map_err(|e| format!("pattern: {e}"))?;

        let mut stdout = io::stdout().lock();
        if args.count {
            emit_count(&mut stdout, matched.len()).map_err(|e| format!("writing output: {e}"))?;
        } else {
            emit_matches(&mut stdout, &lines, &matched, args.line_number)
                .map_err(|e| format!("writing output: {e}"))?;
        }
        return Ok(if matched.is_empty() {
            EXIT_NO_MATCH
        } else {
            EXIT_MATCH
        });
    }

    // TUI mode.
    let initial_pattern = args.pattern.unwrap_or_default();
    let app = FilterApp::new(lines, &initial_pattern, options);
    let (final_app, outcome) = run::run_tui(app).map_err(|e| format!("tui: {e}"))?;

    match outcome {
        Outcome::Emit => {
            let mut stdout = io::stdout().lock();
            emit_matches(&mut stdout, &final_app.lines, &final_app.matched, false)
                .map_err(|e| format!("writing output: {e}"))?;
            Ok(if final_app.matched.is_empty() {
                EXIT_NO_MATCH
            } else {
                EXIT_MATCH
            })
        }
        Outcome::Discard => Ok(EXIT_NO_MATCH),
        Outcome::Pending => Ok(EXIT_ERROR),
    }
}
