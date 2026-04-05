//! PCRE2 step-through debugger using AUTO_CALLOUT.
//!
//! All unsafe FFI code for the debugger is contained in this module.

use super::{EngineError, EngineFlags, EngineResult};

/// A single step in the regex engine's execution trace.
#[derive(Debug, Clone)]
pub struct DebugStep {
    pub index: usize,
    pub pattern_offset: usize,
    pub pattern_item_length: usize,
    pub subject_offset: usize,
    pub is_backtrack: bool,
    pub captures: Vec<Option<(usize, usize)>>,
    pub match_attempt: usize,
}

#[derive(Debug, Clone)]
pub struct PatternToken {
    pub start: usize,
    pub end: usize,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct DebugTrace {
    pub steps: Vec<DebugStep>,
    pub truncated: bool,
    pub offset_map: Vec<PatternToken>,
    pub heatmap: Vec<u32>,
    pub match_attempts: usize,
}

use regex_syntax::ast::parse::Parser;
use regex_syntax::ast::Ast;

pub fn build_offset_map(pattern: &str) -> Vec<PatternToken> {
    let ast = match Parser::new().parse(pattern) {
        Ok(ast) => ast,
        Err(_) => return Vec::new(),
    };
    let mut tokens = Vec::new();
    collect_tokens(&ast, &mut tokens);
    tokens.sort_by_key(|t| t.start);
    tokens.dedup_by_key(|t| t.start);
    tokens
}

fn collect_tokens(ast: &Ast, tokens: &mut Vec<PatternToken>) {
    match ast {
        Ast::Empty(_) => {}
        Ast::Flags(f) => {
            tokens.push(PatternToken {
                start: f.span.start.offset,
                end: f.span.end.offset,
                description: "Flags".to_string(),
            });
        }
        Ast::Literal(lit) => {
            tokens.push(PatternToken {
                start: lit.span.start.offset,
                end: lit.span.end.offset,
                description: format!("Literal '{}'", lit.c),
            });
        }
        Ast::Dot(span) => {
            tokens.push(PatternToken {
                start: span.start.offset,
                end: span.end.offset,
                description: "Any character".to_string(),
            });
        }
        Ast::Assertion(a) => {
            tokens.push(PatternToken {
                start: a.span.start.offset,
                end: a.span.end.offset,
                description: format!("{:?}", a.kind),
            });
        }
        Ast::ClassUnicode(c) => {
            tokens.push(PatternToken {
                start: c.span.start.offset,
                end: c.span.end.offset,
                description: "Unicode class".to_string(),
            });
        }
        Ast::ClassPerl(c) => {
            let name = match c.kind {
                regex_syntax::ast::ClassPerlKind::Digit => "Digit (\\d)",
                regex_syntax::ast::ClassPerlKind::Space => "Whitespace (\\s)",
                regex_syntax::ast::ClassPerlKind::Word => "Word char (\\w)",
            };
            tokens.push(PatternToken {
                start: c.span.start.offset,
                end: c.span.end.offset,
                description: name.to_string(),
            });
        }
        Ast::ClassBracketed(c) => {
            tokens.push(PatternToken {
                start: c.span.start.offset,
                end: c.span.end.offset,
                description: "Character class".to_string(),
            });
        }
        Ast::Repetition(rep) => {
            collect_tokens(&rep.ast, tokens);
        }
        Ast::Group(group) => {
            collect_tokens(&group.ast, tokens);
        }
        Ast::Alternation(alt) => {
            for a in &alt.asts {
                collect_tokens(a, tokens);
            }
        }
        Ast::Concat(concat) => {
            for a in &concat.asts {
                collect_tokens(a, tokens);
            }
        }
    }
}

pub fn find_token_at_offset(offset_map: &[PatternToken], offset: usize) -> Option<usize> {
    for (i, token) in offset_map.iter().enumerate() {
        if offset >= token.start && offset < token.end {
            return Some(i);
        }
    }
    if offset_map.is_empty() {
        return None;
    }
    let mut best = 0;
    let mut best_dist = usize::MAX;
    for (i, token) in offset_map.iter().enumerate() {
        let dist = if offset < token.start {
            token.start - offset
        } else {
            offset - token.end
        };
        if dist < best_dist {
            best_dist = dist;
            best = i;
        }
    }
    Some(best)
}

// Suppress unused import warnings for items that will be used in future tasks.
const _: fn() = || {
    let _: EngineError;
    let _: EngineFlags;
    let _: EngineResult<()>;
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_map_simple_literal() {
        let tokens = build_offset_map("abc");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].start, 0);
        assert_eq!(tokens[0].end, 1);
        assert_eq!(tokens[1].start, 1);
        assert_eq!(tokens[1].end, 2);
        assert_eq!(tokens[2].start, 2);
        assert_eq!(tokens[2].end, 3);
    }

    #[test]
    fn test_offset_map_char_class() {
        let tokens = build_offset_map(r"[a-z]+");
        assert!(!tokens.is_empty());
        assert_eq!(tokens[0].start, 0);
    }

    #[test]
    fn test_offset_map_groups() {
        let tokens = build_offset_map(r"(\d{3})-(\d{4})");
        assert!(!tokens.is_empty());
        let hyphen = tokens.iter().find(|t| t.description.contains('-'));
        assert!(hyphen.is_some());
    }

    #[test]
    fn test_offset_map_empty_pattern() {
        let tokens = build_offset_map("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_find_token_at_offset_basic() {
        let tokens = build_offset_map("abc");
        assert_eq!(find_token_at_offset(&tokens, 0), Some(0));
        assert_eq!(find_token_at_offset(&tokens, 1), Some(1));
        assert_eq!(find_token_at_offset(&tokens, 2), Some(2));
    }

    #[test]
    fn test_find_token_at_offset_empty() {
        let tokens: Vec<PatternToken> = Vec::new();
        assert_eq!(find_token_at_offset(&tokens, 0), None);
    }
}
