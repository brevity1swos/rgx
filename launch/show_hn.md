# Show HN Post (v2 — March 2026)

## Title

Show HN: rgx – terminal regex debugger with JSON output, colored matching, and 3 engines

## URL

https://github.com/brevity1swos/rgx

## Text

I built a TUI for testing regex patterns without leaving the terminal. Just shipped v0.8.0 with batch-mode features for pipeline use.

What it does:

- Live matching that updates on every keystroke
- 3 regex engines (Rust regex, fancy-regex, PCRE2) — switch with Ctrl+E to compare behavior
- Capture group highlighting with distinct colors per group
- Plain-English explanations generated from the regex AST
- Replace/substitution with live preview
- JSON output: `echo "error at line 42" | rgx -p '(\d+)' --json` — structured match data with positions and capture groups, pipe into jq
- Colored output: `--color always` highlights matches inline like grep --color
- Shell completions for bash/zsh/fish
- Ctrl+U exports your current state as a regex101.com URL — iterate locally, share the link
- Project workspaces: `rgx -w parse_urls.toml` saves/loads regex state to a file you can track in git
- Non-interactive batch mode with proper exit codes (0/1/2) — fits into shell pipelines

Written in Rust with ratatui. Install: `cargo install rgx-cli` or `brew install brevity1swos/tap/rgx`.

To be clear about what this isn't: regex101.com is more capable overall — more engines, step-through debugging, shareable links, community patterns. rgx doesn't try to replace it. Ctrl+U actually bridges the two — build your pattern in the terminal, share it via regex101.

Where rgx fills a gap: testing patterns on remote servers where you can't open a browser, piping structured match data into other commands, and testing against the actual Rust `regex` crate behavior (which regex101 doesn't support).

Would love feedback on what would actually be useful for your workflow.
