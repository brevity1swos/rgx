//! Minimal dotted/indexed path language for JSONL field extraction.
//!
//! Grammar:
//!   path    := segment+
//!   segment := ('.' ident) | ('[' digits ']')
//!   ident   := [A-Za-z_][A-Za-z0-9_]*
//!
//! The grammar is deliberately small — no wildcards, no filters, no quoted
//! keys. It is just enough to address a field inside a typical JSONL record
//! (e.g. `.msg`, `.steps[0].text`). Anything more expressive than this is out
//! of scope for the v1 `--json` flag.

use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Segment {
    Key(String),
    Index(usize),
}

/// Parse a dotted/indexed path expression into a list of segments.
///
/// Returns `Err` with a message pointing at the character offset on failure.
pub fn parse_path(s: &str) -> Result<Vec<Segment>, String> {
    if s.is_empty() {
        return Err("empty path".to_string());
    }

    let bytes = s.as_bytes();
    let mut segments = Vec::new();
    let mut i = 0;

    while i < bytes.len() {
        match bytes[i] {
            b'.' => {
                i += 1;
                let start = i;
                if i >= bytes.len() {
                    return Err(format!("expected identifier at position {i}"));
                }
                // First char of identifier must be [A-Za-z_].
                if !is_ident_start(bytes[i]) {
                    return Err(format!(
                        "expected identifier start at position {i}, found {:?}",
                        bytes[i] as char
                    ));
                }
                i += 1;
                while i < bytes.len() && is_ident_continue(bytes[i]) {
                    i += 1;
                }
                // Safe to slice — identifier chars are ASCII.
                let ident = &s[start..i];
                segments.push(Segment::Key(ident.to_string()));
            }
            b'[' => {
                i += 1;
                let start = i;
                while i < bytes.len() && bytes[i].is_ascii_digit() {
                    i += 1;
                }
                if start == i {
                    return Err(format!("expected digits at position {start}"));
                }
                let digits = &s[start..i];
                if i >= bytes.len() || bytes[i] != b']' {
                    return Err(format!("expected ']' at position {i}"));
                }
                let index: usize = digits
                    .parse()
                    .map_err(|e| format!("invalid index {digits:?}: {e}"))?;
                i += 1; // consume ']'
                segments.push(Segment::Index(index));
            }
            other => {
                return Err(format!(
                    "expected '.' or '[' at position {i}, found {:?}",
                    other as char
                ));
            }
        }
    }

    if segments.is_empty() {
        return Err("empty path".to_string());
    }
    Ok(segments)
}

/// Walk a JSON `Value` along the given `path`. Returns `None` if any segment
/// misses (wrong type, missing key, out-of-bounds index).
pub fn extract<'a>(value: &'a Value, path: &[Segment]) -> Option<&'a Value> {
    let mut cur = value;
    for seg in path {
        match seg {
            Segment::Key(k) => cur = cur.as_object()?.get(k)?,
            Segment::Index(i) => cur = cur.as_array()?.get(*i)?,
        }
    }
    Some(cur)
}

fn is_ident_start(b: u8) -> bool {
    b.is_ascii_alphabetic() || b == b'_'
}

fn is_ident_continue(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_'
}
