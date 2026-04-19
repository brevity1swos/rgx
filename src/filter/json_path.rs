//! Minimal dotted/indexed path language for JSONL field extraction.
//!
//! Grammar:
//!
//! ```text
//! path    := segment+
//! segment := ('.' ident) | ('[' digits ']') | ('[' quoted ']')
//! ident   := [A-Za-z_][A-Za-z0-9_]*
//! quoted  := '"' ( [^"\\] | '\\' ( '"' | '\\' ) )* '"'
//! ```
//!
//! Dotted identifiers cover the common JSONL shape (`.msg`, `.steps[0].text`).
//! For keys that can't be expressed as a bare identifier — hyphens, dots,
//! spaces, unicode — use the bracketed-string form: `.["user-id"]`,
//! `["weird key"]`, or `.["日本語"]`. Only `\"` and `\\` are recognized as
//! escapes inside the quoted form; anything else is still on the no-wildcards,
//! no-filters side of scope for `--json`.

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
                        peek_char(s, i)
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
                if i >= bytes.len() {
                    return Err(format!("expected digits or quoted key at position {i}"));
                }
                if bytes[i] == b'"' {
                    // Quoted string key — supports hyphens, dots, spaces, unicode.
                    let (key, consumed) = parse_quoted_key(&bytes[i..], i)?;
                    i += consumed;
                    if i >= bytes.len() || bytes[i] != b']' {
                        return Err(format!("expected ']' at position {i}"));
                    }
                    i += 1; // consume ']'
                    segments.push(Segment::Key(key));
                } else {
                    let start = i;
                    while i < bytes.len() && bytes[i].is_ascii_digit() {
                        i += 1;
                    }
                    if start == i {
                        return Err(format!("expected digits or quoted key at position {start}"));
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
            }
            _ => {
                return Err(format!(
                    "expected '.' or '[' at position {i}, found {:?}",
                    peek_char(s, i)
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

/// Read the first char at `byte_offset`, or `'?'` if the offset is past the
/// end (should never happen where we call this — only there to satisfy the
/// Option). Using `chars().next()` avoids the "one byte looks like æ when it's
/// really the first byte of 日" misreporting that `bytes[i] as char` produces.
fn peek_char(s: &str, byte_offset: usize) -> char {
    s[byte_offset..].chars().next().unwrap_or('?')
}

/// Parse a quoted key starting at the leading `"`. `offset` is the absolute
/// position of that leading `"` in the full source (for error messages).
/// Returns `(key, consumed_bytes)` where `consumed_bytes` includes both
/// quotes. Recognizes `\"` and `\\`; any other backslash pair is a parse error
/// (keeps us honest — we don't silently pass through `\n` etc.).
fn parse_quoted_key(bytes: &[u8], offset: usize) -> Result<(String, usize), String> {
    debug_assert!(!bytes.is_empty() && bytes[0] == b'"');
    let mut out: Vec<u8> = Vec::new();
    let mut j = 1; // skip opening "
    while j < bytes.len() {
        match bytes[j] {
            b'"' => {
                // String is UTF-8 because the original `s` was &str.
                let key = String::from_utf8(out)
                    .map_err(|e| format!("invalid utf-8 in quoted key at {offset}: {e}"))?;
                return Ok((key, j + 1));
            }
            b'\\' => {
                if j + 1 >= bytes.len() {
                    return Err(format!("unterminated escape at position {}", offset + j));
                }
                match bytes[j + 1] {
                    b'"' => out.push(b'"'),
                    b'\\' => out.push(b'\\'),
                    other => {
                        return Err(format!(
                            "unknown escape '\\{}' at position {}",
                            other as char,
                            offset + j
                        ));
                    }
                }
                j += 2;
            }
            b => {
                out.push(b);
                j += 1;
            }
        }
    }
    Err(format!(
        "unterminated quoted key starting at position {offset}"
    ))
}
