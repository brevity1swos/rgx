<div align="center">

# rgx

**regex101, but in your terminal**

[![CI](https://github.com/brevity1swos/rgx/actions/workflows/ci.yml/badge.svg)](https://github.com/brevity1swos/rgx/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/rgx-cli.svg)](https://crates.io/crates/rgx-cli)
[![Downloads](https://img.shields.io/crates/d/rgx-cli.svg)](https://crates.io/crates/rgx-cli)
[![License](https://img.shields.io/crates/l/rgx.svg)](LICENSE-MIT)

A terminal regex debugger with real-time matching, capture group highlighting, and plain-English explanations. Written in Rust.

![demo](https://raw.githubusercontent.com/brevity1swos/rgx/main/assets/demo.gif)

</div>

---

## Features

- **Real-time matching** — matches update on every keystroke
- **3 regex engines** — Rust `regex` (default), `fancy-regex` (lookaround/backrefs), PCRE2 (full features)
- **Capture group highlighting** — distinct colors per group, nested group support
- **Plain-English explanations** — walks the regex AST to generate human-readable breakdowns
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
```

## Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `Tab` | Switch between pattern / test string |
| `Ctrl+E` | Cycle regex engine |
| `Alt+i` | Toggle case-insensitive |
| `Alt+m` | Toggle multi-line |
| `Alt+s` | Toggle dot-matches-newline |
| `Alt+u` | Toggle unicode mode |
| `Alt+x` | Toggle extended mode |
| `?` | Show help |
| `Esc` | Quit |

## Engines

| Engine | Features | Dependencies |
|--------|----------|-------------|
| **Rust regex** (default) | Fast, linear time, Unicode | Pure Rust |
| **fancy-regex** | + lookaround, backreferences | Pure Rust |
| **PCRE2** | + possessive quantifiers, recursion, conditionals | Requires libpcre2 |

## Comparison

| Feature | rgx | regex-tui | rexi | regex101.com |
|---------|:---:|:---------:|:----:|:------------:|
| Real-time matching | Yes | Yes | Yes | Yes |
| Multiple engines | 3 | 2 | 1 | 8 |
| Capture group highlighting | Yes | No | No | Yes |
| Plain-English explanations | Yes | No | No | Yes |
| Regex flags toggle | Yes | Yes | No | Yes |
| Stdin pipe support | Yes | Yes | Yes | No |
| Offline / no browser | Yes | Yes | Yes | No |
| Cross-platform binary | Yes | Yes | No | N/A |

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
