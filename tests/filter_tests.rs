use std::io::Cursor;

use clap::Parser;
use rgx::config::cli::{Cli, Command};
use rgx::filter::{filter_lines, read_input, FilterOptions};

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
