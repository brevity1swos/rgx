# r/commandline Post

## Title

rgx — regex101 for your terminal (real-time matching, 3 engines, replace mode, explanations)

## Body

I built **rgx**, a terminal regex debugger that brings the regex101.com experience to the command line.

**GitHub:** https://github.com/brevity1swos/rgx

**Install:**
```
# Cargo
cargo install rgx-cli

# Homebrew
brew install brevity1swos/tap/rgx

# Shell installer
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/brevity1swos/rgx/releases/latest/download/rgx-installer.sh | sh
```

### Features

- Real-time matching — updates on every keystroke
- 3 regex engines (Rust regex, fancy-regex, PCRE2) — switch between them to compare behavior
- Capture group highlighting with named group support
- Plain-English explanations of your regex pattern
- Replace/substitution mode with live preview
- Pattern syntax highlighting
- Undo/redo and pattern history browsing (Alt+Up/Down)
- Whitespace visualization toggle (Ctrl+W)
- Mouse support (click to focus, scroll to navigate)
- Multi-page help/cheat sheet (F1)
- Pipe text from stdin: `echo "test 123" | rgx '\d+'`

### How it compares

| Feature | rgx | regex-tui | rexi | regex101.com |
|---------|:---:|:---------:|:----:|:------------:|
| Engines | 3 | 2 | 1 | 8 |
| Capture groups | Yes | No | No | Yes |
| Explanations | Yes | No | No | Yes |
| Replace mode | Yes | No | No | Yes |
| Syntax highlighting | Yes | No | No | Yes |
| Mouse support | Yes | No | No | N/A |
| Offline | Yes | Yes | Yes | No |

Cross-platform (Linux, macOS, Windows). Written in Rust. Feedback welcome!
