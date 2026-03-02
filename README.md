<div align="center">

# rgx

**A terminal regex tester with real-time matching and multi-engine support**

[![CI](https://github.com/brevity1swos/rgx/actions/workflows/ci.yml/badge.svg)](https://github.com/brevity1swos/rgx/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/rgx-cli.svg)](https://crates.io/crates/rgx-cli)
[![Downloads](https://img.shields.io/crates/d/rgx-cli.svg)](https://crates.io/crates/rgx-cli)
[![License](https://img.shields.io/crates/l/rgx.svg)](LICENSE-MIT)

Test and debug regular expressions without leaving your terminal. Written in Rust.

![demo](https://raw.githubusercontent.com/brevity1swos/rgx/main/assets/demo.gif?v=3)

*Press F1 in the app for a multi-page cheat sheet with keyboard shortcuts, regex syntax, and engine-specific features.*

</div>

---

## Who is this for?

rgx is useful if you:

- **Work on remote servers** where opening a browser isn't practical — SSH sessions, containers, air-gapped environments
- **Want to pipe regex results** into other commands (`echo "log" | rgx '\d+' | ...`) — regex101 can't do this
- **Need to test against specific engine behavior** — check if your pattern works in Rust's `regex` crate vs PCRE2 without guessing
- **Prefer staying in the terminal** and find the context switch to a browser tab disruptive

If you write regex a few times a month and regex101.com works fine for you, it probably still will. rgx is strongest for developers who do regex-heavy work in terminal-centric workflows.

## Features

- **Real-time matching** — matches update on every keystroke
- **Syntax-highlighted pattern input** — AST-based coloring for groups, quantifiers, character classes, anchors, and escapes
- **3 regex engines** — Rust `regex` (default), `fancy-regex` (lookaround/backrefs), PCRE2 (full features)
- **Capture group highlighting** — distinct colors per group, nested group support
- **Plain-English explanations** — walks the regex AST to generate human-readable breakdowns
- **Replace/substitution mode** — live preview with `$1`, `${name}`, `$0`/`$&` syntax
- **Match detail + clipboard** — navigate matches/captures with Up/Down, copy with Ctrl+Y
- **Pattern history + undo** — Ctrl+Z/Ctrl+Shift+Z undo/redo, Alt+Up/Down browse history
- **Context-sensitive cheat sheet** — F1 multi-page help: shortcuts, regex syntax, engine-specific features
- **Whitespace visualization** — toggle with Ctrl+W to show spaces as `·`, newlines as `↵`, tabs as `→`
- **Mouse support** — click to focus/position cursor, scroll to navigate panels
- **Engine selector** — switch engines with Ctrl+E, see where behavior differs
- **Regex flags** — toggle case-insensitive, multiline, dotall, unicode, extended
- **Stdin pipe support** — `echo "test string" | rgx '\d+'`
- **Cross-platform** — Linux, macOS, Windows

## Installation

### From crates.io

```bash
cargo install rgx-cli
```

### From prebuilt binaries

Download from [GitHub Releases](https://github.com/brevity1swos/rgx/releases/latest).

### Shell installer

```bash
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/brevity1swos/rgx/releases/latest/download/rgx-installer.sh | sh
```

### Homebrew

```bash
brew install brevity1swos/tap/rgx
```

<details>
<summary>More installation methods</summary>

### From source

```bash
git clone https://github.com/brevity1swos/rgx.git
cd rgx
cargo install --path .
```

### Without PCRE2 (zero C dependencies)

```bash
cargo install rgx-cli --no-default-features
```

</details>

## Usage

```bash
# Interactive mode
rgx

# Start with a pattern
rgx '\d{3}-\d{3}-\d{4}'

# Pipe text from stdin
echo "Call 555-123-4567 or 555-987-6543" | rgx '\d{3}-\d{3}-\d{4}'

# Use a specific engine
rgx --engine fancy '\w+(?=@)'

# With flags
rgx -i 'hello'

# With replacement template
rgx -r '$2/$1' '(\w+)@(\w+)'
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Tab` | Cycle focus: pattern / test / replace / matches / explanation |
| `Up/Down` | Scroll panel / move cursor / select match |
| `Enter` | Insert newline (test string) |
| `Ctrl+E` | Cycle regex engine |
| `Ctrl+Z` | Undo |
| `Ctrl+Shift+Z` | Redo |
| `Ctrl+Y` | Copy selected match to clipboard |
| `Ctrl+W` | Toggle whitespace visualization |
| `Ctrl+Left/Right` | Move cursor by word |
| `Alt+Up/Down` | Browse pattern history |
| `Alt+i/m/s/u/x` | Toggle flags (case, multiline, dotall, unicode, extended) |
| `F1` | Show help (Left/Right to page through) |
| `Mouse click` | Focus panel and position cursor |
| `Mouse scroll` | Scroll panel under cursor |
| `Esc` | Quit |

## Engines

| Engine | Features | Dependencies |
|--------|----------|-------------|
| **Rust regex** (default) | Fast, linear time, Unicode | Pure Rust |
| **fancy-regex** | + lookaround, backreferences | Pure Rust |
| **PCRE2** | + possessive quantifiers, recursion, conditionals | Requires libpcre2 |

## Comparison

### vs. terminal alternatives

| Feature | rgx | regex-tui | rexi |
|---------|:---:|:---------:|:----:|
| Real-time matching | Yes | Yes | Yes |
| Multiple engines | 3 | 2 | 1 |
| Capture group highlighting | Yes | No | No |
| Plain-English explanations | Yes | No | No |
| Replace/substitution | Yes | No | No |
| Match clipboard copy | Yes | No | No |
| Undo/redo | Yes | No | No |
| Whitespace visualization | Yes | Yes | No |
| Mouse support | Yes | No | No |
| Regex flags toggle | Yes | Yes | No |
| Stdin pipe support | Yes | Yes | Yes |

### vs. regex101.com

regex101.com is the more capable tool overall — it has 8 engines, step-through debugging, code generation, shareable permalinks, and a community pattern library. rgx doesn't try to replace it. Where rgx is useful instead:

- **Offline/remote work** — no browser or internet needed
- **Pipeline integration** — pipe stdin in, pipe results out with `Ctrl+O`
- **Engine-specific testing** — test against Rust's `regex` crate directly (regex101 doesn't have this engine)
- **Workspace save/restore** — save your session and pick up later

## Configuration

rgx looks for a config file at `~/.config/rgx/config.toml`:

```toml
default_engine = "rust"  # "rust", "fancy", or "pcre2"
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
