use std::io::Cursor;

use clap::Parser;
use rgx::config::cli::{Cli, Command};
use rgx::filter::{
    emit_count, emit_matches, extract_strings, filter_lines, filter_lines_with_extracted,
    read_input, FilterApp, FilterOptions, Outcome,
};

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
fn filter_subcommand_with_json_flag_parses() {
    let cli = Cli::try_parse_from(["rgx", "filter", "--json", ".msg", "boom"]).unwrap();
    match cli.command {
        Some(Command::Filter(args)) => {
            assert_eq!(args.json.as_deref(), Some(".msg"));
            assert_eq!(args.pattern.as_deref(), Some("boom"));
        }
        _ => panic!("expected Filter subcommand"),
    }
}

#[test]
fn filter_subcommand_without_json_flag_defaults_to_none() {
    let cli = Cli::try_parse_from(["rgx", "filter", "pat"]).unwrap();
    match cli.command {
        Some(Command::Filter(args)) => {
            assert!(args.json.is_none());
        }
        _ => panic!("expected Filter subcommand"),
    }
}

#[test]
fn extract_strings_happy_path() {
    let lines = to_lines(&[
        r#"{"msg":"hello"}"#,
        r#"{"msg":"world"}"#,
        r#"{"msg":"boom"}"#,
    ]);
    let got = extract_strings(&lines, ".msg").unwrap();
    assert_eq!(
        got,
        vec![
            Some("hello".to_string()),
            Some("world".to_string()),
            Some("boom".to_string()),
        ]
    );
}

#[test]
fn extract_strings_skips_parse_failure() {
    let lines = to_lines(&[
        r#"{"msg":"ok"}"#,
        "this is not json",
        r#"{"msg":"also-ok"}"#,
    ]);
    let got = extract_strings(&lines, ".msg").unwrap();
    assert_eq!(
        got,
        vec![Some("ok".to_string()), None, Some("also-ok".to_string()),]
    );
}

#[test]
fn extract_strings_skips_non_string_value() {
    let lines = to_lines(&[r#"{"n":42}"#, r#"{"n":"forty-two"}"#]);
    let got = extract_strings(&lines, ".n").unwrap();
    // Only the string value survives; the integer is None.
    assert_eq!(got, vec![None, Some("forty-two".to_string())]);
}

#[test]
fn extract_strings_skips_missing_path() {
    let lines = to_lines(&[r#"{"other":"x"}"#, r#"{"msg":"found"}"#]);
    let got = extract_strings(&lines, ".msg").unwrap();
    assert_eq!(got, vec![None, Some("found".to_string())]);
}

#[test]
fn filter_lines_with_extracted_matches_extracted_values() {
    // Raw lines are irrelevant; the extracted field is what gets matched.
    let extracted = vec![
        Some("hello".to_string()),
        Some("boom".to_string()),
        Some("goodbye".to_string()),
    ];
    let got = filter_lines_with_extracted(&extracted, "^b", FilterOptions::default()).unwrap();
    assert_eq!(got, vec![1]);
}

#[test]
fn filter_lines_with_extracted_skips_none_entries() {
    let extracted = vec![
        Some("keep".to_string()),
        None, // parse failure / missing path / non-string
        Some("other".to_string()),
    ];
    // Pattern `.` would match any non-empty string. None lines must still be skipped.
    let got = filter_lines_with_extracted(&extracted, ".", FilterOptions::default()).unwrap();
    assert_eq!(got, vec![0, 2]);
}

#[test]
fn filter_lines_with_extracted_empty_pattern_passes_present_values() {
    let extracted = vec![Some("a".to_string()), None, Some("b".to_string())];
    let got = filter_lines_with_extracted(&extracted, "", FilterOptions::default()).unwrap();
    assert_eq!(got, vec![0, 2]);
}

#[test]
fn filter_lines_with_extracted_invert_skips_none() {
    // In invert mode, None entries still don't emit. Only present-and-non-matching
    // values qualify.
    let extracted = vec![Some("match".to_string()), None, Some("other".to_string())];
    let options = FilterOptions {
        invert: true,
        case_insensitive: false,
    };
    let got = filter_lines_with_extracted(&extracted, "match", options).unwrap();
    assert_eq!(got, vec![2]);
}

#[test]
fn filter_lines_with_extracted_invalid_pattern_errors() {
    let extracted = vec![Some("x".to_string())];
    assert!(
        filter_lines_with_extracted(&extracted, "(unclosed", FilterOptions::default()).is_err()
    );
}

#[test]
fn extract_strings_propagates_parse_path_error() {
    let lines = to_lines(&[r#"{"msg":"x"}"#]);
    let err = extract_strings(&lines, "not-a-path").unwrap_err();
    assert!(!err.is_empty(), "error message should not be empty");
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
    let (got, truncated) = read_input(None, Cursor::new(data), 100_000).unwrap();
    assert_eq!(got, vec!["foo", "bar", "baz"]);
    assert!(!truncated);
}

#[test]
fn read_input_handles_invalid_utf8() {
    // Stray \xFF\xFE bytes between valid UTF-8 lines — grep tolerates these;
    // we now do too. Each invalid byte becomes U+FFFD.
    let data: &[u8] = b"valid\n\xFF\xFEinvalid\nok\n";
    let (got, truncated) = read_input(None, Cursor::new(data), 100_000).unwrap();
    assert!(!truncated);
    assert_eq!(got.len(), 3);
    assert_eq!(got[0], "valid");
    assert!(
        got[1].contains('\u{FFFD}'),
        "middle line should have replacement char, got {:?}",
        got[1]
    );
    assert!(got[1].ends_with("invalid"));
    assert_eq!(got[2], "ok");
}

#[test]
fn read_input_from_file() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("input.txt");
    std::fs::write(&path, "alpha\nbeta\n").unwrap();
    let (got, truncated) = read_input(Some(&path), Cursor::new("ignored"), 100_000).unwrap();
    assert_eq!(got, vec!["alpha", "beta"]);
    assert!(!truncated);
}

#[test]
fn read_input_caps_at_max_lines() {
    // Feed 1000 lines with a cap of 5 — expect exactly 5 lines back and
    // truncated == true.
    let data: String = (0..1000).map(|i| format!("line-{i}\n")).collect();
    let (got, truncated) = read_input(None, Cursor::new(data), 5).unwrap();
    assert_eq!(got.len(), 5);
    assert_eq!(got[0], "line-0");
    assert_eq!(got[4], "line-4");
    assert!(truncated, "cap was hit mid-stream; should be flagged");
}

#[test]
fn read_input_exact_fit_not_truncated() {
    // When the input fits exactly within max_lines, no truncation warning.
    let data = "a\nb\nc\n";
    let (got, truncated) = read_input(None, Cursor::new(data), 3).unwrap();
    assert_eq!(got, vec!["a", "b", "c"]);
    assert!(!truncated);
}

#[test]
fn read_input_zero_means_no_cap() {
    // max_lines = 0 disables the cap entirely.
    let data: String = (0..50).map(|i| format!("l{i}\n")).collect();
    let (got, truncated) = read_input(None, Cursor::new(data), 0).unwrap();
    assert_eq!(got.len(), 50);
    assert!(!truncated);
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

#[test]
fn filter_app_empty_pattern_shows_all_lines() {
    let lines = to_lines(&["one", "two", "three"]);
    let app = FilterApp::new(lines, "", FilterOptions::default());
    assert_eq!(app.matched, vec![0, 1, 2]);
    assert_eq!(app.outcome, Outcome::Pending);
    assert!(app.error.is_none());
}

#[test]
fn filter_app_applies_initial_pattern() {
    let lines = to_lines(&["error 1", "ok", "error 2"]);
    let app = FilterApp::new(lines, "error", FilterOptions::default());
    assert_eq!(app.matched, vec![0, 2]);
}

#[test]
fn filter_app_invalid_pattern_sets_error() {
    let lines = to_lines(&["a"]);
    let app = FilterApp::new(lines, "(unclosed", FilterOptions::default());
    assert!(app.error.is_some());
    assert!(app.matched.is_empty());
}

#[test]
fn filter_app_toggle_invert_flips_match_set() {
    let lines = to_lines(&["error 1", "ok", "error 2"]);
    let mut app = FilterApp::new(lines, "error", FilterOptions::default());
    assert_eq!(app.matched, vec![0, 2]);
    app.toggle_invert();
    assert_eq!(app.matched, vec![1]);
}

#[test]
fn filter_app_toggle_case_insensitive_recomputes() {
    let lines = to_lines(&["ERROR one", "ok", "error two"]);
    let mut app = FilterApp::new(lines.clone(), "error", FilterOptions::default());
    assert_eq!(app.matched, vec![2]);
    app.toggle_case_insensitive();
    assert_eq!(app.matched, vec![0, 2]);
}

#[test]
fn filter_app_selection_clamps_on_pattern_change() {
    let lines = to_lines(&["a", "b", "c", "d"]);
    let mut app = FilterApp::new(lines, "", FilterOptions::default());
    app.selected = 3;
    // Change pattern — now only one line matches.
    app.pattern_editor = rgx::input::editor::Editor::with_content("a".to_string());
    app.recompute();
    assert_eq!(app.matched, vec![0]);
    assert_eq!(app.selected, 0);
}

#[test]
fn filter_app_populates_match_spans() {
    let lines = to_lines(&["a1b22", "nope"]);
    let app = FilterApp::new(lines, r"\d+", FilterOptions::default());
    assert_eq!(app.matched, vec![0]);
    assert_eq!(app.match_spans.len(), 1);
    assert_eq!(app.match_spans[0], vec![1..2, 3..5]);
}

#[test]
fn filter_app_with_json_matches_extracted_field() {
    // Three JSONL lines — match against the `msg` field only.
    let lines = to_lines(&[
        r#"{"level":"info","msg":"hello"}"#,
        r#"{"level":"error","msg":"boom"}"#,
        r#"{"level":"info","msg":"goodbye"}"#,
    ]);
    let extracted = extract_strings(&lines, ".msg").unwrap();
    let app = FilterApp::with_json_extracted(lines, extracted, "boom", FilterOptions::default());
    // Only the second line has a msg that matches `boom`.
    assert_eq!(app.matched, vec![1]);
}

#[test]
fn filter_app_with_json_skips_parse_failures() {
    let lines = to_lines(&[
        r#"{"msg":"ok"}"#,
        "this is not json",
        r#"{"msg":"also-ok"}"#,
    ]);
    let extracted = extract_strings(&lines, ".msg").unwrap();
    // Pattern `.` would match any non-empty string — the bad line must be
    // excluded because its extracted value is None.
    let app = FilterApp::with_json_extracted(lines, extracted, ".", FilterOptions::default());
    assert_eq!(app.matched, vec![0, 2]);
}

#[test]
fn filter_app_match_spans_refer_to_extracted_string() {
    // The raw line has `msg` at index >20, but the match span should be
    // computed within the extracted string "boom" — so `oo` lives at 1..3.
    let lines = to_lines(&[r#"{"level":"error","msg":"boom"}"#]);
    let extracted = extract_strings(&lines, ".msg").unwrap();
    let app = FilterApp::with_json_extracted(lines, extracted, "oo", FilterOptions::default());
    assert_eq!(app.matched, vec![0]);
    assert_eq!(app.match_spans, vec![vec![1..3]]);
}

#[test]
fn filter_app_with_json_empty_pattern_shows_only_parseable_lines() {
    let lines = to_lines(&[r#"{"msg":"ok"}"#, "nope", r#"{"msg":"also"}"#]);
    let extracted = extract_strings(&lines, ".msg").unwrap();
    let app = FilterApp::with_json_extracted(lines, extracted, "", FilterOptions::default());
    assert_eq!(app.matched, vec![0, 2]);
}

#[test]
fn filter_app_with_json_invert_skips_none() {
    let lines = to_lines(&[r#"{"msg":"match"}"#, "not json", r#"{"msg":"other"}"#]);
    let extracted = extract_strings(&lines, ".msg").unwrap();
    let app = FilterApp::with_json_extracted(
        lines,
        extracted,
        "match",
        FilterOptions {
            invert: true,
            case_insensitive: false,
        },
    );
    // Only line 2 qualifies: its extracted value exists AND doesn't match.
    // The "not json" line is still excluded even in invert mode.
    assert_eq!(app.matched, vec![2]);
}

#[test]
fn filter_app_match_spans_empty_in_invert_mode() {
    // Invert mode emits lines that didn't match — there's nothing to highlight.
    let lines = to_lines(&["error 1", "ok", "error 2"]);
    let app = FilterApp::new(
        lines,
        r"\d+",
        FilterOptions {
            invert: true,
            case_insensitive: false,
        },
    );
    assert_eq!(app.matched, vec![1]);
    assert_eq!(app.match_spans, vec![Vec::<std::ops::Range<usize>>::new()]);
}

#[test]
fn filter_ui_highlights_match_spans_with_match_bg() {
    // A pattern that matches — verify at least one cell in the match list
    // has the MATCH_BG background color applied.
    use ratatui::backend::TestBackend;
    use ratatui::style::Color;
    use ratatui::Terminal;

    let mut terminal = Terminal::new(TestBackend::new(80, 10)).unwrap();
    let lines = to_lines(&["abc123def"]);
    let app = FilterApp::new(lines, r"\d+", FilterOptions::default());
    terminal
        .draw(|frame| rgx::filter::ui::render(frame, &app))
        .unwrap();
    let buf = terminal.backend().buffer().clone();

    let match_bg = Color::Rgb(69, 71, 90); // theme::MATCH_BG
    let mut found_highlighted = false;
    for y in 0..buf.area.height {
        for x in 0..buf.area.width {
            let cell = &buf[(x, y)];
            if cell.bg == match_bg {
                // Match background cells should correspond to digit characters.
                let sym = cell.symbol();
                if sym == "1" || sym == "2" || sym == "3" {
                    found_highlighted = true;
                }
            }
        }
    }
    assert!(
        found_highlighted,
        "expected at least one cell with MATCH_BG covering a digit"
    );
}

#[test]
fn filter_ui_render_does_not_panic() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
    let lines = to_lines(&["alpha", "beta", "gamma"]);
    let app = FilterApp::new(lines, "a", FilterOptions::default());
    terminal
        .draw(|frame| rgx::filter::ui::render(frame, &app))
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let rendered: String = buf
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect::<Vec<_>>()
        .join("");
    assert!(rendered.contains("Pattern"));
    assert!(rendered.contains("Matches"));
    assert!(rendered.contains("alpha"));
    assert!(rendered.contains("gamma"));
}

#[test]
fn handle_key_enter_sets_emit() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use rgx::filter::run::handle_key;
    let lines = to_lines(&["x"]);
    let mut app = FilterApp::new(lines, "x", FilterOptions::default());
    handle_key(&mut app, KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    assert_eq!(app.outcome, Outcome::Emit);
    assert!(app.should_quit);
}

#[test]
fn handle_key_esc_sets_discard() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use rgx::filter::run::handle_key;
    let lines = to_lines(&["x"]);
    let mut app = FilterApp::new(lines, "x", FilterOptions::default());
    handle_key(&mut app, KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert_eq!(app.outcome, Outcome::Discard);
    assert!(app.should_quit);
}

#[test]
fn handle_key_alt_v_toggles_invert() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use rgx::filter::run::handle_key;
    let lines = to_lines(&["error", "ok"]);
    let mut app = FilterApp::new(lines, "error", FilterOptions::default());
    assert_eq!(app.matched, vec![0]);
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('v'), KeyModifiers::ALT),
    );
    assert_eq!(app.matched, vec![1]);
}

#[test]
fn handle_key_alt_i_toggles_case() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use rgx::filter::run::handle_key;
    let lines = to_lines(&["ERROR", "ok"]);
    let mut app = FilterApp::new(lines, "error", FilterOptions::default());
    assert!(app.matched.is_empty());
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('i'), KeyModifiers::ALT),
    );
    assert_eq!(app.matched, vec![0]);
}

#[test]
fn handle_key_typing_refilters() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use rgx::filter::run::handle_key;
    let lines = to_lines(&["alpha", "beta", "gamma"]);
    let mut app = FilterApp::new(lines, "", FilterOptions::default());
    assert_eq!(app.matched.len(), 3);
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('a'), KeyModifiers::NONE),
    );
    // Pattern is now "a" — matches alpha, beta, gamma all contain 'a'.
    assert_eq!(app.matched.len(), 3);
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE),
    );
    // Pattern is "al" — only alpha matches.
    assert_eq!(app.matched, vec![0]);
}

#[test]
fn handle_key_backspace_refilters() {
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use rgx::filter::run::handle_key;
    let lines = to_lines(&["alpha", "beta", "gamma"]);
    let mut app = FilterApp::new(lines, "al", FilterOptions::default());
    assert_eq!(app.matched, vec![0]);
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Backspace, KeyModifiers::NONE),
    );
    // Back to "a" — all three match.
    assert_eq!(app.matched.len(), 3);
}

#[test]
fn handle_key_plain_q_inserts_into_pattern_not_quit() {
    // Regression: 'q' as an exit shortcut prevented users from typing patterns
    // like `quote`, `sequence`, or `\bq\w+`. Esc and Ctrl+C still handle exit.
    use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
    use rgx::filter::run::handle_key;
    let lines = to_lines(&["quick brown fox"]);
    let mut app = FilterApp::new(lines, "", FilterOptions::default());
    handle_key(
        &mut app,
        KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
    );
    assert!(
        !app.should_quit,
        "plain 'q' must not quit — it belongs in the pattern"
    );
    assert_eq!(app.pattern(), "q");
    // The pattern "q" matches the single line.
    assert_eq!(app.matched, vec![0]);
}

#[test]
fn filter_ui_render_scrolls_selection_into_view() {
    // Regression: selection could scroll past the visible pane when the match
    // list was longer than the viewport. Now the render function derives a
    // start offset that always keeps `selected` visible.
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let lines: Vec<String> = (0..50).map(|i| format!("line-{i:02}")).collect();
    let mut app = FilterApp::new(lines, "line", FilterOptions::default());
    app.selected = 45;

    // 10-row viewport: match pane is rows 3..9 (6 rows inner after borders+pattern+status).
    let mut terminal = Terminal::new(TestBackend::new(60, 10)).unwrap();
    terminal
        .draw(|frame| rgx::filter::ui::render(frame, &app))
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let rendered: String = buf
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect::<Vec<_>>()
        .join("");
    assert!(
        rendered.contains("line-45"),
        "selected row (line-45) must be visible at bottom of pane"
    );
    assert!(
        !rendered.contains("line-00"),
        "viewport should have scrolled past the top — line-00 must not be visible"
    );
}

#[test]
fn filter_ui_render_with_invalid_pattern_shows_error() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;
    let mut terminal = Terminal::new(TestBackend::new(80, 20)).unwrap();
    let lines = to_lines(&["a"]);
    let app = FilterApp::new(lines, "(unclosed", FilterOptions::default());
    terminal
        .draw(|frame| rgx::filter::ui::render(frame, &app))
        .unwrap();
    let buf = terminal.backend().buffer().clone();
    let rendered: String = buf
        .content()
        .iter()
        .map(|c| c.symbol())
        .collect::<Vec<_>>()
        .join("");
    assert!(rendered.contains("invalid"));
    assert!(rendered.contains("error"));
}

mod json_path_tests {
    use rgx::filter::json_path::{parse_path, Segment};

    #[test]
    fn parse_path_single_key() {
        assert_eq!(
            parse_path(".msg").unwrap(),
            vec![Segment::Key("msg".into())]
        );
    }

    #[test]
    fn parse_path_nested() {
        assert_eq!(
            parse_path(".a.b.c").unwrap(),
            vec![
                Segment::Key("a".into()),
                Segment::Key("b".into()),
                Segment::Key("c".into()),
            ]
        );
    }

    #[test]
    fn parse_path_index() {
        assert_eq!(
            parse_path(".items[0]").unwrap(),
            vec![Segment::Key("items".into()), Segment::Index(0)]
        );
    }

    #[test]
    fn parse_path_mixed() {
        assert_eq!(
            parse_path(".steps[1].text").unwrap(),
            vec![
                Segment::Key("steps".into()),
                Segment::Index(1),
                Segment::Key("text".into()),
            ]
        );
    }

    #[test]
    fn parse_path_empty_returns_err() {
        assert!(parse_path("").is_err());
    }

    #[test]
    fn parse_path_missing_dot_errors() {
        // `msg` without a leading `.` is not a valid expression.
        assert!(parse_path("msg").is_err());
    }

    #[test]
    fn parse_path_unclosed_bracket_errors() {
        assert!(parse_path(".items[0").is_err());
    }

    #[test]
    fn parse_path_non_numeric_index_errors() {
        assert!(parse_path(".items[abc]").is_err());
    }

    #[test]
    fn parse_path_identifier_with_underscores_and_digits() {
        assert_eq!(
            parse_path(".field_1.a2b").unwrap(),
            vec![Segment::Key("field_1".into()), Segment::Key("a2b".into()),]
        );
    }

    #[test]
    fn parse_path_identifier_starting_with_digit_errors() {
        // Identifiers may not start with a digit.
        assert!(parse_path(".1field").is_err());
    }

    // --- extract() tests ---

    use rgx::filter::json_path::extract;
    use serde_json::json;

    #[test]
    fn extract_top_level_field() {
        let v = json!({"a": 1});
        let path = parse_path(".a").unwrap();
        assert_eq!(extract(&v, &path), Some(&json!(1)));
    }

    #[test]
    fn extract_nested_field() {
        let v = json!({"a": {"b": "x"}});
        let path = parse_path(".a.b").unwrap();
        assert_eq!(extract(&v, &path), Some(&json!("x")));
    }

    #[test]
    fn extract_array_index() {
        let v = json!({"items": ["x", "y"]});
        let path = parse_path(".items[1]").unwrap();
        assert_eq!(extract(&v, &path), Some(&json!("y")));
    }

    #[test]
    fn extract_missing_key_returns_none() {
        let v = json!({"a": 1});
        let path = parse_path(".b").unwrap();
        assert_eq!(extract(&v, &path), None);
    }

    #[test]
    fn extract_out_of_bounds_index_returns_none() {
        let v = json!({"items": ["x"]});
        let path = parse_path(".items[5]").unwrap();
        assert_eq!(extract(&v, &path), None);
    }

    #[test]
    fn extract_type_mismatch_returns_none() {
        // Asking for an array index on a string value must not panic.
        let v = json!({"items": "not-an-array"});
        let path = parse_path(".items[0]").unwrap();
        assert_eq!(extract(&v, &path), None);
    }

    #[test]
    fn extract_mixed_path_on_realistic_value() {
        let v = json!({
            "steps": [
                {"text": "hello"},
                {"text": "world"},
            ]
        });
        let path = parse_path(".steps[1].text").unwrap();
        assert_eq!(extract(&v, &path), Some(&json!("world")));
    }
}

mod cli_e2e {
    use std::io::Write as _;
    use std::process::{Command, Stdio};

    fn rgx_bin() -> std::path::PathBuf {
        // Cargo puts integration test binaries next to the main binary under target/debug.
        let mut p = std::env::current_exe().unwrap();
        p.pop(); // test binary name
        if p.ends_with("deps") {
            p.pop();
        }
        p.push(if cfg!(windows) { "rgx.exe" } else { "rgx" });
        p
    }

    #[test]
    fn cli_filter_count_reads_stdin() {
        let bin = rgx_bin();
        assert!(bin.exists(), "rgx binary not found at {bin:?}; build first");
        let mut child = Command::new(&bin)
            .args(["filter", "--count", r"\d+"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"error 1\nok\nerror 2\nwarn\n")
            .unwrap();
        let out = child.wait_with_output().unwrap();
        assert_eq!(out.status.code(), Some(0));
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "2");
    }

    #[test]
    fn cli_filter_emit_matching_lines_from_file() {
        let bin = rgx_bin();
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("log.txt");
        std::fs::write(&path, "info: ok\nerror: boom\ninfo: ok2\nerror: kaboom\n").unwrap();
        let out = Command::new(&bin)
            .args(["filter", "-f", path.to_str().unwrap(), "-n", "error"])
            .stderr(Stdio::piped())
            .output()
            .unwrap();
        assert_eq!(out.status.code(), Some(0));
        assert_eq!(
            String::from_utf8_lossy(&out.stdout),
            "2:error: boom\n4:error: kaboom\n"
        );
    }

    #[test]
    fn cli_filter_no_match_returns_exit_1() {
        let bin = rgx_bin();
        let mut child = Command::new(&bin)
            .args(["filter", "--count", "zzz"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(b"foo\nbar\n")
            .unwrap();
        let out = child.wait_with_output().unwrap();
        assert_eq!(out.status.code(), Some(1));
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "0");
    }

    #[test]
    fn cli_filter_invalid_pattern_returns_exit_2() {
        let bin = rgx_bin();
        let mut child = Command::new(&bin)
            .args(["filter", "--count", "(unclosed"])
            .stdin(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child.stdin.as_mut().unwrap().write_all(b"foo\n").unwrap();
        let out = child.wait_with_output().unwrap();
        assert_eq!(out.status.code(), Some(2));
    }

    #[test]
    fn cli_filter_json_extracts_and_matches() {
        let bin = rgx_bin();
        let input = concat!(
            r#"{"level":"info","msg":"hello"}"#,
            "\n",
            r#"{"level":"error","msg":"boom"}"#,
            "\n",
            r#"{"level":"info","msg":"goodbye"}"#,
            "\n",
            "bad line not json\n",
        );
        let mut child = Command::new(&bin)
            .args(["filter", "--json", ".msg", "--count", "^b"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        assert_eq!(out.status.code(), Some(0));
        // Only "boom" matches ^b; "hello" and "goodbye" don't; bad line skipped.
        assert_eq!(String::from_utf8_lossy(&out.stdout).trim(), "1");
    }

    #[test]
    fn cli_filter_json_emits_raw_line_not_extracted() {
        // When --json is set and the pattern matches, the RAW JSON line is
        // emitted — not the extracted value.
        let bin = rgx_bin();
        let input = concat!(
            r#"{"level":"info","msg":"hello"}"#,
            "\n",
            r#"{"level":"error","msg":"boom"}"#,
            "\n",
        );
        let mut child = Command::new(&bin)
            .args(["filter", "--json", ".msg", "-n", "boom"])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();
        child
            .stdin
            .as_mut()
            .unwrap()
            .write_all(input.as_bytes())
            .unwrap();
        let out = child.wait_with_output().unwrap();
        assert_eq!(out.status.code(), Some(0));
        assert_eq!(
            String::from_utf8_lossy(&out.stdout),
            "2:{\"level\":\"error\",\"msg\":\"boom\"}\n"
        );
    }
}
