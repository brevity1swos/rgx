use regex_syntax::ast::Ast;

use super::formatter;
use super::ExplainNode;

#[derive(Default)]
pub struct ExplainVisitor {
    nodes: Vec<ExplainNode>,
    depth: usize,
}

impl ExplainVisitor {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn into_nodes(self) -> Vec<ExplainNode> {
        self.nodes
    }

    fn push(&mut self, description: String) {
        self.nodes.push(ExplainNode {
            depth: self.depth,
            description,
        });
    }

    pub fn visit(&mut self, ast: &Ast) {
        match ast {
            Ast::Empty(_) => {}
            Ast::Flags(flags) => {
                self.push(formatter::format_flags_item(&flags.flags));
            }
            Ast::Literal(lit) => {
                self.push(formatter::format_literal(lit));
            }
            Ast::Dot(_) => {
                self.push("Any character (except newline by default)".to_string());
            }
            Ast::Assertion(assertion) => {
                self.push(formatter::format_assertion(assertion));
            }
            Ast::ClassUnicode(class) => {
                self.push(formatter::format_unicode_class(class));
            }
            Ast::ClassPerl(class) => {
                self.push(formatter::format_perl_class(class));
            }
            Ast::ClassBracketed(class) => {
                self.push(formatter::format_bracketed_class(class));
            }
            Ast::Repetition(rep) => {
                self.push(formatter::format_repetition(rep));
                self.depth += 1;
                self.visit(&rep.ast);
                self.depth -= 1;
            }
            Ast::Group(group) => {
                self.push(formatter::format_group(group));
                self.depth += 1;
                self.visit(&group.ast);
                self.depth -= 1;
            }
            Ast::Alternation(alt) => {
                self.push("Either:".to_string());
                self.depth += 1;
                for (i, ast) in alt.asts.iter().enumerate() {
                    if i > 0 {
                        self.push("Or:".to_string());
                    }
                    self.visit(ast);
                }
                self.depth -= 1;
            }
            Ast::Concat(concat) => {
                for ast in &concat.asts {
                    self.visit(ast);
                }
            }
        }
    }
}
