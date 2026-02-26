# r/commandline Post

## Title

rgx — a TUI regex debugger with live matching, 3 engines, and replace mode

## Body

I built **rgx** because I was tired of alt-tabbing to regex101.com every time I needed to test a pattern.

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
- Pipe from stdin: `echo "test 123" | rgx '\d+'`
- Undo/redo, pattern history (Alt+Up/Down), whitespace visualization (Ctrl+W), mouse support

Cross-platform (Linux, macOS, Windows). Single binary, no runtime dependencies.

Feedback welcome — especially on what would make this more useful in your daily workflow.
