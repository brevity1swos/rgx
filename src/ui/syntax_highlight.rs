use ratatui::{
    style::{Color, Style},
    text::Span,
};
use regex_syntax::ast::{Ast, LiteralKind};

use super::theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxCategory {
    Literal,
    Group,
    Quantifier,
    CharClass,
    Anchor,
    Escape,
    Alternation,
}

#[derive(Debug, Clone)]
pub struct SyntaxToken {
    pub start: usize,
    pub end: usize,
    pub category: SyntaxCategory,
}

pub fn highlight(pattern: &str) -> Vec<SyntaxToken> {
    if pattern.is_empty() {
        return vec![];
    }

    let ast = match crate::explain::parse_ast(pattern) {
        Some(ast) => ast,
        None => return vec![],
    };

    let mut tokens = Vec::new();
    collect_tokens(&ast, &mut tokens);
    tokens.sort_by_key(|t| t.start);
    tokens
}

fn collect_tokens(ast: &Ast, tokens: &mut Vec<SyntaxToken>) {
    match ast {
        Ast::Empty(_) => {}
        Ast::Literal(lit) => {
            let start = lit.span.start.offset;
            let end = lit.span.end.offset;
            let category = match lit.kind {
                LiteralKind::Verbatim => SyntaxCategory::Literal,
                _ => SyntaxCategory::Escape,
            };
            tokens.push(SyntaxToken {
                start,
                end,
                category,
            });
        }
        Ast::Dot(span) => {
            tokens.push(SyntaxToken {
                start: span.start.offset,
                end: span.end.offset,
                category: SyntaxCategory::Anchor,
            });
        }
        Ast::Assertion(a) => {
            tokens.push(SyntaxToken {
                start: a.span.start.offset,
                end: a.span.end.offset,
                category: SyntaxCategory::Anchor,
            });
        }
        Ast::ClassPerl(c) => {
            tokens.push(SyntaxToken {
                start: c.span.start.offset,
                end: c.span.end.offset,
                category: SyntaxCategory::CharClass,
            });
        }
        Ast::ClassUnicode(c) => {
            tokens.push(SyntaxToken {
                start: c.span.start.offset,
                end: c.span.end.offset,
                category: SyntaxCategory::CharClass,
            });
        }
        Ast::ClassBracketed(c) => {
            tokens.push(SyntaxToken {
                start: c.span.start.offset,
                end: c.span.end.offset,
                category: SyntaxCategory::CharClass,
            });
        }
        Ast::Repetition(rep) => {
            collect_tokens(&rep.ast, tokens);
            tokens.push(SyntaxToken {
                start: rep.op.span.start.offset,
                end: rep.op.span.end.offset,
                category: SyntaxCategory::Quantifier,
            });
        }
        Ast::Group(group) => {
            // Token for the opening delimiter (everything up to the inner AST)
            let group_start = group.span.start.offset;
            let inner_start = group.ast.span().start.offset;
            let inner_end = group.ast.span().end.offset;
            let group_end = group.span.end.offset;

            if inner_start > group_start {
                tokens.push(SyntaxToken {
                    start: group_start,
                    end: inner_start,
                    category: SyntaxCategory::Group,
                });
            }

            collect_tokens(&group.ast, tokens);

            // Token for the closing delimiter
            if group_end > inner_end {
                tokens.push(SyntaxToken {
                    start: inner_end,
                    end: group_end,
                    category: SyntaxCategory::Group,
                });
            }
        }
        Ast::Alternation(alt) => {
            // Visit children and derive `|` positions from gaps between siblings
            for (i, child) in alt.asts.iter().enumerate() {
                collect_tokens(child, tokens);
                if i + 1 < alt.asts.len() {
                    let pipe_start = child.span().end.offset;
                    let next_start = alt.asts[i + 1].span().start.offset;
                    // The `|` should be between the end of this child and start of next
                    if next_start > pipe_start {
                        tokens.push(SyntaxToken {
                            start: pipe_start,
                            end: pipe_start + 1,
                            category: SyntaxCategory::Alternation,
                        });
                    }
                }
            }
        }
        Ast::Concat(concat) => {
            for child in &concat.asts {
                collect_tokens(child, tokens);
            }
        }
        Ast::Flags(flags) => {
            tokens.push(SyntaxToken {
                start: flags.span.start.offset,
                end: flags.span.end.offset,
                category: SyntaxCategory::Group,
            });
        }
    }
}

pub fn category_color(cat: SyntaxCategory) -> Color {
    match cat {
        SyntaxCategory::Literal => theme::TEXT,
        SyntaxCategory::Group => theme::BLUE,
        SyntaxCategory::Quantifier => theme::MAUVE,
        SyntaxCategory::CharClass => theme::GREEN,
        SyntaxCategory::Anchor => theme::TEAL,
        SyntaxCategory::Escape => theme::PEACH,
        SyntaxCategory::Alternation => theme::YELLOW,
    }
}

pub fn build_highlighted_spans<'a>(pattern: &'a str, tokens: &[SyntaxToken]) -> Vec<Span<'a>> {
    let mut spans = Vec::new();
    let mut pos = 0;

    for token in tokens {
        // Skip overlapping or out-of-order tokens
        if token.start < pos {
            continue;
        }
        // Gap before this token → plain text
        if token.start > pos {
            spans.push(Span::styled(
                &pattern[pos..token.start],
                Style::default().fg(theme::TEXT),
            ));
        }
        let end = token.end.min(pattern.len());
        if end > token.start {
            spans.push(Span::styled(
                &pattern[token.start..end],
                Style::default().fg(category_color(token.category)),
            ));
        }
        pos = end;
    }

    // Remaining text after last token
    if pos < pattern.len() {
        spans.push(Span::styled(
            &pattern[pos..],
            Style::default().fg(theme::TEXT),
        ));
    }

    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_pattern() {
        assert!(highlight("").is_empty());
    }

    #[test]
    fn test_invalid_pattern_returns_empty() {
        assert!(highlight("(unclosed").is_empty());
    }

    #[test]
    fn test_literal_only() {
        let tokens = highlight("hello");
        assert!(!tokens.is_empty());
        for t in &tokens {
            assert_eq!(t.category, SyntaxCategory::Literal);
        }
    }

    #[test]
    fn test_perl_class() {
        let tokens = highlight(r"\d+");
        let categories: Vec<_> = tokens.iter().map(|t| t.category).collect();
        assert!(categories.contains(&SyntaxCategory::CharClass));
        assert!(categories.contains(&SyntaxCategory::Quantifier));
    }

    #[test]
    fn test_group_and_quantifier() {
        let tokens = highlight(r"(\w+)");
        let categories: Vec<_> = tokens.iter().map(|t| t.category).collect();
        assert!(categories.contains(&SyntaxCategory::Group));
        assert!(categories.contains(&SyntaxCategory::CharClass));
        assert!(categories.contains(&SyntaxCategory::Quantifier));
    }

    #[test]
    fn test_alternation() {
        let tokens = highlight("foo|bar");
        let categories: Vec<_> = tokens.iter().map(|t| t.category).collect();
        assert!(categories.contains(&SyntaxCategory::Alternation));
        assert!(categories.contains(&SyntaxCategory::Literal));
    }

    #[test]
    fn test_anchors() {
        let tokens = highlight(r"^hello$");
        let categories: Vec<_> = tokens.iter().map(|t| t.category).collect();
        assert!(categories.contains(&SyntaxCategory::Anchor));
        assert!(categories.contains(&SyntaxCategory::Literal));
    }

    #[test]
    fn test_escape_sequences() {
        let tokens = highlight(r"\n\t");
        for t in &tokens {
            assert_eq!(t.category, SyntaxCategory::Escape);
        }
    }

    #[test]
    fn test_dot() {
        let tokens = highlight("a.b");
        let categories: Vec<_> = tokens.iter().map(|t| t.category).collect();
        assert!(categories.contains(&SyntaxCategory::Anchor));
    }

    #[test]
    fn test_bracketed_class() {
        let tokens = highlight("[a-z]+");
        let categories: Vec<_> = tokens.iter().map(|t| t.category).collect();
        assert!(categories.contains(&SyntaxCategory::CharClass));
        assert!(categories.contains(&SyntaxCategory::Quantifier));
    }

    #[test]
    fn test_build_highlighted_spans_covers_full_pattern() {
        let pattern = r"(\w+)@(\w+)";
        let tokens = highlight(pattern);
        let spans = build_highlighted_spans(pattern, &tokens);
        let reconstructed: String = spans.iter().map(|s| s.content.as_ref()).collect();
        assert_eq!(reconstructed, pattern);
    }

    #[test]
    fn test_lazy_quantifier() {
        let tokens = highlight(r"\d+?");
        let quant: Vec<_> = tokens
            .iter()
            .filter(|t| t.category == SyntaxCategory::Quantifier)
            .collect();
        assert_eq!(quant.len(), 1);
        // Should cover both `+` and `?`
        assert_eq!(quant[0].end - quant[0].start, 2);
    }

    #[test]
    fn test_named_group() {
        let tokens = highlight(r"(?P<name>\w+)");
        let groups: Vec<_> = tokens
            .iter()
            .filter(|t| t.category == SyntaxCategory::Group)
            .collect();
        // Opening `(?P<name>` and closing `)`
        assert_eq!(groups.len(), 2);
    }
}
