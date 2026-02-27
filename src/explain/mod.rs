pub mod formatter;
pub mod visitor;

use regex_syntax::ast::parse::Parser;
use visitor::ExplainVisitor;

#[derive(Debug, Clone)]
pub struct ExplainNode {
    pub depth: usize,
    pub description: String,
}

pub fn explain(pattern: &str) -> Result<Vec<ExplainNode>, (String, Option<usize>)> {
    if pattern.is_empty() {
        return Ok(vec![]);
    }

    let ast = Parser::new().parse(pattern).map_err(|e| {
        let offset = pattern[..e.span().start.offset].chars().count();
        (format!("Parse error: {e}"), Some(offset))
    })?;

    let mut visitor = ExplainVisitor::new();
    visitor.visit(&ast);
    Ok(visitor.into_nodes())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_pattern() {
        let result = explain("").unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn test_simple_literal() {
        let result = explain("hello").unwrap();
        assert!(!result.is_empty());
    }

    #[test]
    fn test_digit_class() {
        let result = explain(r"\d+").unwrap();
        assert!(!result.is_empty());
        let text: String = result.iter().map(|n| n.description.clone()).collect();
        assert!(text.to_lowercase().contains("digit"));
    }

    #[test]
    fn test_capture_group() {
        let result = explain(r"(\w+)@(\w+)").unwrap();
        assert!(!result.is_empty());
        let text: String = result.iter().map(|n| n.description.clone()).collect();
        assert!(text.to_lowercase().contains("group"));
    }

    #[test]
    fn test_invalid_pattern() {
        let result = explain(r"(unclosed");
        assert!(result.is_err());
        let (msg, offset) = result.unwrap_err();
        assert!(msg.contains("Parse error"));
        assert!(offset.is_some());
    }
}
