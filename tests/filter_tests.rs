use std::io::Cursor;

use clap::Parser;
use rgx::config::cli::{Cli, Command};
use rgx::filter::{emit_count, emit_matches, filter_lines, read_input, FilterOptions};

fn to_lines(strs: &[&str]) -> Vec<String> {
    strs.iter().map(|s| s.to_string()).collect()
}

#[test]
fn filter_subcommand_with_pattern_parses() {
    let cli = Cli::try_parse_from(["rgx", "filter", "error"]).unwrap();
    match cli.command {
        Some(Command::Filter(args)) => {
            assert_eq!(args.pattern.as_deref(), Some("error"));
            assert!(!args.invert);
            assert!(!args.count);
            assert!(!args.line_number);
        }
        _ => panic!("expected Filter subcommand"),
    }
}

#[test]
fn filter_subcommand_with_flags_parses() {
    let cli =
        Cli::try_parse_from(["rgx", "filter", "-vc", "-n", "-f", "log.txt", "error"]).unwrap();
    match cli.command {
        Some(Command::Filter(args)) => {
            assert!(args.invert);
            assert!(args.count);
            assert!(args.line_number);
            assert_eq!(
                args.file.as_deref().and_then(|p| p.to_str()),
                Some("log.txt")
            );
            assert_eq!(args.pattern.as_deref(), Some("error"));
        }
        _ => panic!("expected Filter subcommand"),
    }
}

#[test]
fn bare_rgx_has_no_subcommand() {
    let cli = Cli::try_parse_from(["rgx"]).unwrap();
    assert!(cli.command.is_none());
}

#[test]
fn empty_pattern_passes_every_line() {
    let lines = to_lines(&["foo", "bar", "baz"]);
    let got = filter_lines(&lines, "", FilterOptions::default()).unwrap();
    assert_eq!(got, vec![0, 1, 2]);
}

#[test]
fn empty_pattern_with_invert_passes_nothing() {
    let lines = to_lines(&["foo", "bar", "baz"]);
    let got = filter_lines(
        &lines,
        "",
        FilterOptions {
            invert: true,
            case_insensitive: false,
        },
    )
    .unwrap();
    assert!(got.is_empty());
}

#[test]
fn simple_pattern_selects_matching_lines() {
    let lines = to_lines(&["hello 42", "world", "hello 99", "foo"]);
    let got = filter_lines(&lines, r"\d+", FilterOptions::default()).unwrap();
    assert_eq!(got, vec![0, 2]);
}

#[test]
fn invert_flag_selects_non_matching_lines() {
    let lines = to_lines(&["hello 42", "world", "hello 99", "foo"]);
    let got = filter_lines(
        &lines,
        r"\d+",
        FilterOptions {
            invert: true,
            case_insensitive: false,
        },
    )
    .unwrap();
    assert_eq!(got, vec![1, 3]);
}

#[test]
fn case_insensitive_flag() {
    let lines = to_lines(&["Error: boom", "OK", "ERROR again"]);
    let got = filter_lines(
        &lines,
        "error",
        FilterOptions {
            invert: false,
            case_insensitive: true,
        },
    )
    .unwrap();
    assert_eq!(got, vec![0, 2]);
}

#[test]
fn invalid_pattern_returns_err() {
    let lines = to_lines(&["a"]);
    let got = filter_lines(&lines, "(unclosed", FilterOptions::default());
    assert!(got.is_err());
}

#[test]
fn read_input_from_in_memory_stdin() {
    let data = "foo\nbar\nbaz\n";
    let got = read_input(None, Cursor::new(data)).unwrap();
    assert_eq!(got, vec!["foo", "bar", "baz"]);
}

#[test]
fn read_input_from_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("input.txt");
    std::fs::write(&path, "alpha\nbeta\n").unwrap();
    let got = read_input(Some(&path), Cursor::new("ignored")).unwrap();
    assert_eq!(got, vec!["alpha", "beta"]);
}

#[test]
fn emit_matches_plain() {
    let lines = to_lines(&["alpha", "beta", "gamma"]);
    let matched = vec![0, 2];
    let mut buf = Vec::new();
    emit_matches(&mut buf, &lines, &matched, false).unwrap();
    assert_eq!(String::from_utf8(buf).unwrap(), "alpha\ngamma\n");
}

#[test]
fn emit_matches_with_line_numbers() {
    let lines = to_lines(&["alpha", "beta", "gamma"]);
    let matched = vec![0, 2];
    let mut buf = Vec::new();
    emit_matches(&mut buf, &lines, &matched, true).unwrap();
    assert_eq!(String::from_utf8(buf).unwrap(), "1:alpha\n3:gamma\n");
}

#[test]
fn emit_count_writes_number() {
    let mut buf = Vec::new();
    emit_count(&mut buf, 7).unwrap();
    assert_eq!(String::from_utf8(buf).unwrap(), "7\n");
}

#[test]
fn count_mode_returns_expected_count() {
    let lines = to_lines(&["one 1", "two", "three 3", "four 4"]);
    let options = FilterOptions::default();
    let matched = filter_lines(&lines, r"\d", options).unwrap();
    let mut buf = Vec::new();
    emit_count(&mut buf, matched.len()).unwrap();
    assert_eq!(String::from_utf8(buf).unwrap(), "3\n");
}
