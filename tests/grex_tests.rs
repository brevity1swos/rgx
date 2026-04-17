use rgx::grex_integration::{generate, GrexOptions};

#[test]
fn default_options_match_spec_defaults() {
    let opts = GrexOptions::default();
    assert!(opts.digit);
    assert!(opts.anchors);
    assert!(!opts.case_insensitive);
}

#[test]
fn empty_input_returns_empty_string() {
    let result = generate(&[], GrexOptions::default());
    assert_eq!(result, "");
}
