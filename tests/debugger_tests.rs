#![cfg(feature = "pcre2-engine")]

use rgx::engine::pcre2_debug::debug_match;
use rgx::engine::EngineFlags;

#[test]
fn test_debug_match_end_to_end() {
    let flags = EngineFlags::default();
    let trace = debug_match(r"\d+", "abc 123 def", &flags, 10000, 0).unwrap();
    assert!(!trace.steps.is_empty());
    assert!(!trace.offset_map.is_empty());
    assert_eq!(trace.heatmap.len(), trace.offset_map.len());
}

#[test]
fn test_catastrophic_backtracking_detection() {
    let flags = EngineFlags::default();
    // (?:a|aa)+b forces the engine to explore multiple ways to partition a run
    // of a's before matching b, generating many callout steps and a hot heatmap.
    let trace = debug_match("(?:a|aa)+b", "aaaaaab", &flags, 50000, 0).unwrap();
    assert!(
        trace.steps.len() > 10,
        "expected many steps, got {}",
        trace.steps.len()
    );
    let max_heat = trace.heatmap.iter().copied().max().unwrap_or(0);
    assert!(max_heat > 1, "expected hot heatmap, max was {}", max_heat);
}

#[test]
fn test_debug_with_flags() {
    let flags = EngineFlags {
        case_insensitive: true,
        ..Default::default()
    };
    let trace = debug_match("abc", "ABC", &flags, 10000, 0).unwrap();
    assert!(!trace.steps.is_empty(), "case-insensitive should match");
}

#[test]
fn test_debug_unicode_pattern() {
    let flags = EngineFlags {
        unicode: true,
        ..Default::default()
    };
    let trace = debug_match(r"\w+", "hello", &flags, 10000, 0).unwrap();
    assert!(!trace.steps.is_empty());
}
