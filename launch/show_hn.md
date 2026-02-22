# Show HN Post

## Title

Show HN: rgx – regex101 for the terminal, with 3 engines and plain-English explanations

## URL

https://github.com/brevity1swos/rgx

## Text

I built a terminal regex debugger that aims to be what regex101.com is for the browser — but offline, fast, and keyboard-driven.

Features:

- Real-time matching that updates on every keystroke
- 3 regex engines: Rust regex (default), fancy-regex (lookaround/backrefs), PCRE2 (full features) — switch with Ctrl+E to see where behavior differs
- Capture group highlighting with distinct colors per group and named group display
- Plain-English explanations — walks the regex AST to generate human-readable breakdowns like "Match one or more digits, followed by a literal '-'"
- Replace/substitution mode with live preview ($1, ${name}, $0/$& syntax)
- Pattern syntax highlighting (groups, quantifiers, character classes, anchors all color-coded)
- Undo/redo, pattern history (Alt+Up/Down to browse previous patterns)
- Whitespace visualization (toggle with Ctrl+W to see spaces as dots, newlines as arrows)
- Mouse support — click to focus panels, scroll to navigate
- Multi-page cheat sheet (F1) that adapts to the selected engine

Written in Rust with ratatui + crossterm. Install with `cargo install rgx-cli` or `brew install brevity1swos/tap/rgx`.

I built this because I found myself constantly switching between the terminal and browser to test regexes. The killer feature for me is the multi-engine support — being able to quickly check if a pattern works differently between Rust's regex crate and PCRE2 has saved me hours of debugging.

The comparison with other terminal regex tools:

| Feature | rgx | regex-tui | rexi | regex101.com |
|---------|-----|-----------|------|--------------|
| Engines | 3 | 2 | 1 | 8 |
| Capture group highlighting | Yes | No | No | Yes |
| Plain-English explanations | Yes | No | No | Yes |
| Replace/substitution | Yes | No | No | Yes |
| Syntax highlighting | Yes | No | No | Yes |
| Undo/redo | Yes | No | No | Yes |
| Mouse support | Yes | No | No | N/A |
| Offline / no browser | Yes | Yes | Yes | No |

Would love feedback on the UX and what features you'd find most useful.
