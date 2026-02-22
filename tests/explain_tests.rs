use rgx::explain::explain;

#[test]
fn explain_empty() {
    let nodes = explain("").unwrap();
    assert!(nodes.is_empty());
}

#[test]
fn explain_literal() {
    let nodes = explain("hello").unwrap();
    assert!(!nodes.is_empty());
    assert!(nodes.iter().any(|n| n.description.contains("Literal")));
}

#[test]
fn explain_digit() {
    let nodes = explain(r"\d").unwrap();
    assert!(!nodes.is_empty());
    assert!(nodes
        .iter()
        .any(|n| n.description.to_lowercase().contains("digit")));
}

#[test]
fn explain_word_char() {
    let nodes = explain(r"\w").unwrap();
    assert!(nodes
        .iter()
        .any(|n| n.description.to_lowercase().contains("word")));
}

#[test]
fn explain_whitespace() {
    let nodes = explain(r"\s").unwrap();
    assert!(nodes
        .iter()
        .any(|n| n.description.to_lowercase().contains("whitespace")));
}

#[test]
fn explain_quantifiers() {
    let nodes = explain(r"\d+").unwrap();
    assert!(nodes
        .iter()
        .any(|n| n.description.to_lowercase().contains("one or more")));

    let nodes = explain(r"\d*").unwrap();
    assert!(nodes
        .iter()
        .any(|n| n.description.to_lowercase().contains("zero or more")));

    let nodes = explain(r"\d?").unwrap();
    assert!(nodes
        .iter()
        .any(|n| n.description.to_lowercase().contains("optional")));
}

#[test]
fn explain_range_quantifier() {
    let nodes = explain(r"\d{3}").unwrap();
    assert!(nodes.iter().any(|n| n.description.contains("Exactly 3")));

    let nodes = explain(r"\d{2,5}").unwrap();
    assert!(nodes
        .iter()
        .any(|n| n.description.contains("Between 2 and 5")));
}

#[test]
fn explain_capture_group() {
    let nodes = explain(r"(\w+)").unwrap();
    assert!(nodes
        .iter()
        .any(|n| n.description.to_lowercase().contains("capture group")));
}

#[test]
fn explain_named_capture() {
    let nodes = explain(r"(?P<name>\w+)").unwrap();
    assert!(nodes.iter().any(|n| n.description.contains("name")));
}

#[test]
fn explain_alternation() {
    let nodes = explain(r"cat|dog").unwrap();
    assert!(nodes
        .iter()
        .any(|n| n.description.to_lowercase().contains("either")));
}

#[test]
fn explain_anchors() {
    let nodes = explain(r"^\d+$").unwrap();
    assert!(nodes
        .iter()
        .any(|n| n.description.to_lowercase().contains("start")));
    assert!(nodes
        .iter()
        .any(|n| n.description.to_lowercase().contains("end")));
}

#[test]
fn explain_character_class() {
    let nodes = explain(r"[a-zA-Z]").unwrap();
    assert!(nodes
        .iter()
        .any(|n| n.description.to_lowercase().contains("character class")));
}

#[test]
fn explain_dot() {
    let nodes = explain(r".").unwrap();
    assert!(nodes
        .iter()
        .any(|n| n.description.to_lowercase().contains("any character")));
}

#[test]
fn explain_word_boundary() {
    let nodes = explain(r"\b").unwrap();
    assert!(nodes
        .iter()
        .any(|n| n.description.to_lowercase().contains("word boundary")));
}

#[test]
fn explain_lazy_quantifier() {
    let nodes = explain(r"\d+?").unwrap();
    assert!(nodes
        .iter()
        .any(|n| n.description.to_lowercase().contains("lazy")));
}

#[test]
fn explain_invalid_pattern() {
    assert!(explain(r"(unclosed").is_err());
}

#[test]
fn explain_complex_email() {
    let nodes = explain(r"[\w.+-]+@[\w-]+\.[\w.]+").unwrap();
    assert!(!nodes.is_empty());
    // Should contain multiple explanation nodes for this complex pattern
    assert!(nodes.len() > 3);
}
