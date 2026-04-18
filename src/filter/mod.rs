//! `rgx filter` subcommand — live/non-interactive regex filter over stdin or a file.

use std::io::{self, BufRead, BufReader, IsTerminal, Read, Write};
use std::path::Path;

use crate::config::cli::FilterArgs;
use crate::engine::{self, EngineFlags, EngineKind};

pub mod app;
pub mod json_path;
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

/// Returns per-line extracted strings. `None` means the line should be excluded
/// from matching (JSON parse failure, path miss, or non-string value). The
/// returned vector has the same length as `lines`, so callers can index it
/// directly alongside the raw lines.
pub fn extract_strings(lines: &[String], path_expr: &str) -> Result<Vec<Option<String>>, String> {
    let path = json_path::parse_path(path_expr)?;
    let mut out = Vec::with_capacity(lines.len());
    for line in lines {
        let extracted = match serde_json::from_str::<serde_json::Value>(line) {
            Ok(v) => json_path::extract(&v, &path).and_then(|v| v.as_str().map(str::to_string)),
            Err(_) => None,
        };
        out.push(extracted);
    }
    Ok(out)
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
///
/// Invalid UTF-8 bytes are replaced with `U+FFFD REPLACEMENT CHARACTER` rather
/// than aborting the read — this matches `grep`'s behavior and keeps the filter
/// usable against binary-ish logs (e.g. files with stray latin-1 bytes).
///
/// `max_lines` caps the number of lines read to prevent OOM on unbounded
/// streams. Pass `0` to disable the cap. Returns `(lines, truncated)` where
/// `truncated` is `true` if the cap was reached before end-of-input.
pub fn read_input(
    file: Option<&Path>,
    fallback: impl Read,
    max_lines: usize,
) -> io::Result<(Vec<String>, bool)> {
    let mut reader: Box<dyn BufRead> = match file {
        Some(path) => Box::new(BufReader::new(std::fs::File::open(path)?)),
        None => Box::new(BufReader::new(fallback)),
    };
    let mut out = Vec::new();
    let mut buf = Vec::new();
    let mut truncated = false;
    loop {
        if max_lines != 0 && out.len() >= max_lines {
            // Peek: is there any more data after the cap? Only then do we
            // flag truncation, so callers don't warn about files that just
            // happen to have exactly `max_lines` lines.
            buf.clear();
            let n = reader.read_until(b'\n', &mut buf)?;
            if n > 0 {
                truncated = true;
            }
            break;
        }
        buf.clear();
        let n = reader.read_until(b'\n', &mut buf)?;
        if n == 0 {
            break;
        }
        // Strip trailing \n and optional \r.
        let end = buf
            .iter()
            .rposition(|b| *b != b'\n' && *b != b'\r')
            .map(|i| i + 1)
            .unwrap_or(0);
        out.push(String::from_utf8_lossy(&buf[..end]).into_owned());
    }
    Ok((out, truncated))
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
    let (lines, truncated) = read_input(args.file.as_deref(), io::stdin(), args.max_lines)
        .map_err(|e| format!("reading input: {e}"))?;
    if truncated {
        eprintln!(
            "rgx filter: input truncated at {} lines (use --max-lines to override)",
            args.max_lines
        );
    }

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
