# Show HN Post

## Title

Show HN: rgx – a terminal regex debugger with live matching and plain-English explanations

## URL

https://github.com/brevity1swos/rgx

## Text

I kept alt-tabbing between my terminal and regex101.com to test patterns. So I built a TUI that does the same thing — offline, fast, keyboard-driven.

What it does:

- Live matching that updates on every keystroke
- 3 regex engines (Rust regex, fancy-regex, PCRE2) — switch with Ctrl+E to see where behavior differs
- Capture group highlighting with distinct colors per group
- Plain-English explanations generated from the regex AST ("match one or more digits, followed by a literal '-'")
- Replace/substitution mode with live preview
- Pattern syntax highlighting, undo/redo, pattern history, mouse support

Written in Rust with ratatui. Install: `cargo install rgx-cli` or `brew install brevity1swos/tap/rgx`.

The multi-engine switching is the feature I use most — you can instantly check if a lookahead works in Rust's regex crate vs PCRE2 without leaving the terminal.

Would love feedback on the UX and what features would actually be useful for your workflow.
