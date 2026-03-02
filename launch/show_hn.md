# Show HN Post

## Title

Show HN: rgx – a terminal regex tester with live matching and 3 engines

## URL

https://github.com/brevity1swos/rgx

## Text

I built a TUI for testing regex patterns without leaving the terminal. It's most useful if you're working over SSH, in containers, or just prefer not to context-switch to regex101.com.

What it does:

- Live matching that updates on every keystroke
- 3 regex engines (Rust regex, fancy-regex, PCRE2) — switch with Ctrl+E to compare behavior differences
- Capture group highlighting with distinct colors per group
- Plain-English explanations generated from the regex AST
- Replace/substitution mode with live preview
- Pipe from stdin: `echo "log line" | rgx '\d+'` and output results to stdout with Ctrl+O

Written in Rust with ratatui. Install: `cargo install rgx-cli` or `brew install brevity1swos/tap/rgx`.

To be clear about what this isn't: regex101.com is more capable overall — more engines, step-through debugging, shareable links, community patterns. rgx doesn't try to replace it.

Where rgx fills a gap: testing patterns on remote servers where you can't open a browser, piping results into other commands, and testing against the actual Rust `regex` crate behavior (which regex101 doesn't support).

The multi-engine switching is the feature I use most — instantly checking if a lookahead works in Rust's `regex` crate vs PCRE2.

Would love feedback on what would actually be useful for your workflow.
