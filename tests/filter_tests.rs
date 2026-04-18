use std::io::Cursor;

use clap::Parser;
use rgx::config::cli::{Cli, Command};
use rgx::filter::{
    emit_count, emit_matches, filter_lines, read_input, FilterApp, FilterOptions, Outcome,
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
}
