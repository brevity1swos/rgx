<div align="center">

# rgx

**A terminal regex tester with real-time matching and multi-engine support**

[![CI](https://github.com/brevity1swos/rgx/actions/workflows/ci.yml/badge.svg)](https://github.com/brevity1swos/rgx/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/rgx-cli.svg)](https://crates.io/crates/rgx-cli)
[![Downloads](https://img.shields.io/crates/d/rgx-cli.svg)](https://crates.io/crates/rgx-cli)
[![License](https://img.shields.io/crates/l/rgx.svg)](LICENSE-MIT)

Test and debug regular expressions without leaving your terminal. Written in Rust.

![demo](https://raw.githubusercontent.com/brevity1swos/rgx/main/assets/demo.gif?v=6)

*Press F1 in the app for a multi-page cheat sheet.*

</div>

---

## Who is this for?

rgx is useful if you:

- **Work on remote servers** where opening a browser isn't practical — SSH, containers, air-gapped environments.
- **Want to pipe regex results** into other commands (`echo "log" | rgx -p '\d+' | sort`) — regex101 can't do this.
- **Need engine-specific behavior** — check whether a pattern works in Rust's `regex` crate vs. PCRE2 without guessing.
- **Prefer staying in the terminal** and find the context switch to a browser disruptive.

If you write regex a few times a month and regex101.com works fine for you, it probably still will. rgx is strongest for developers doing regex-heavy work in terminal-centric workflows.

## Install

```bash
cargo install rgx-cli                                         # crates.io
brew install brevity1swos/tap/rgx                             # Homebrew
yay -S rgx-cli                                                # AUR
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/brevity1swos/rgx/releases/latest/download/rgx-installer.sh | sh
```

Prebuilt binaries are also on [GitHub Releases](https://github.com/brevity1swos/rgx/releases/latest).

<details>
<summary>Build from source / enable PCRE2</summary>

```bash
# From source
git clone https://github.com/brevity1swos/rgx.git
cd rgx && cargo install --path .

# With PCRE2 engine (requires libpcre2-dev)
cargo install rgx-cli --features pcre2-engine
```

See [docs/advanced.md](docs/advanced.md#building-with-pcre2) for the PCRE2 security note.

</details>

## Quickstart

```bash
rgx                                   # interactive TUI
rgx '\d{3}-\d{3}-\d{4}'               # start with a pattern
echo "Call 555-123-4567" | rgx '\d+'  # stdin as test string
rgx -p -t "error 404" '\d+'           # batch mode (non-interactive)
cat app.jsonl | rgx filter --json '.msg' 'error'   # live stream filter on a JSON field
```

Full flag reference, piping recipes, and `rgx filter` + `--json` usage:
**[docs/usage.md](docs/usage.md)**.

## Features

- Real-time matching with AST-based syntax highlighting and capture-group colors
- **3 regex engines**: Rust `regex` (default), `fancy-regex` (lookaround / backrefs), PCRE2 (+ recursion / conditionals)
- **Auto engine selection** — upgrades engines automatically when your pattern needs lookahead, backreferences, or recursion
- **Plain-English explanations** for any pattern, generated from the AST
- **Step-through debugger** (Ctrl+D, PCRE2) with backtracking visualization and heatmap mode
- **Code generation** — Ctrl+G produces ready-to-paste code in 8 languages (Rust, Python, JS, Go, Java, C#, PHP, Ruby)
- **Generate regex from examples** — Ctrl+X opens a [grex](https://crates.io/crates/grex) overlay
- **Live filter mode** — `rgx filter` streams stdin/file through a regex TUI, with `--json` JSONL-field extraction
- **Non-interactive batch mode** — `-p` with `--count`, `--group`, `--json`, `--color`, grep-like exit codes
- **Vim mode**, **mouse**, **pattern history + undo/redo**, **clipboard copy**, **whitespace visualization**
- **Workspaces** — save/load regex state to a TOML file (`-w`) — track in git
- **Test suite mode** — `rgx --test file.toml` validates patterns against assertions in CI
- **Editor integrations** — VS Code, Neovim, Zed, tmux
- **Shell completions** — `--completions bash|zsh|fish`
- **Cross-platform** — Linux, macOS, Windows

## Documentation

- **[Usage](docs/usage.md)** — interactive / batch / filter modes, all flags, piping recipes
- **[Keyboard shortcuts](docs/shortcuts.md)** — main TUI, vim mode, filter mode
- **[Editor integrations](docs/integrations.md)** — VS Code, Neovim, Zed, tmux
- **[Advanced](docs/advanced.md)** — test suite mode, config file, engines deep-dive, comparison matrix

## Engines at a glance

| Engine | Features | Dependencies |
|--------|----------|--------------|
| **Rust `regex`** (default) | Fast, linear time, Unicode | Pure Rust |
| **fancy-regex** | + lookaround, backreferences | Pure Rust |
| **PCRE2** | + possessive quantifiers, recursion, conditionals | `libpcre2` |

Full matrix and comparison against other tools: [docs/advanced.md](docs/advanced.md#comparison-vs-terminal-alternatives).

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
