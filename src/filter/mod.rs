//! `rgx filter` subcommand — live/non-interactive regex filter over stdin or a file.

use crate::engine::{self, EngineFlags, EngineKind};

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
