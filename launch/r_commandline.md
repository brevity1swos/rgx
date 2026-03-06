# r/commandline Post

## Title

rgx — a TUI regex tester with live matching, 3 engines, and stdin pipe support

## Body

I built **rgx** for testing regex patterns without leaving the terminal.

**GitHub:** https://github.com/brevity1swos/rgx

**Install:**
```
cargo install rgx-cli
brew install brevity1swos/tap/rgx
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/brevity1swos/rgx/releases/latest/download/rgx-installer.sh | sh
```

### What it does

- Live matching — updates as you type
- 3 regex engines (Rust regex, fancy-regex, PCRE2) — Ctrl+E to switch and compare
- Capture group highlighting with named groups
- Plain-English explanations of your pattern
- Replace/substitution with live preview
- Non-interactive batch mode: `echo "test 123" | rgx -p '\d+'` — prints matches to stdout and exits
- Pipeline composability: `cat log | rgx -p 'ERROR: (.*)' | sort | uniq -c`
- Exit codes: 0 = match found, 1 = no match, 2 = error — works with `&&`, `||`, `set -e`
- Interactive output: Ctrl+O outputs results to stdout when leaving the TUI
- Capture pattern: `PATTERN=$(rgx -P)` — edit interactively, capture the final pattern
- Whitespace visualization (Ctrl+W), undo/redo, pattern history

Cross-platform (Linux, macOS, Windows). Single binary.

### Who this is for

Mostly useful if you:
- Work on remote machines where opening a browser isn't practical
- Want regex results piped into other commands
- Need to test patterns against specific engine behavior (e.g., PCRE2 vs Rust regex)

If regex101.com works fine for your workflow, it's the more feature-rich tool overall. rgx fills a gap for terminal-centric use.

Feedback welcome — especially on what would make this more useful for your workflow.
