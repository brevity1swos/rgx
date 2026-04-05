<div align="center">

# rgx

**A terminal regex tester with real-time matching and multi-engine support**

[![CI](https://github.com/brevity1swos/rgx/actions/workflows/ci.yml/badge.svg)](https://github.com/brevity1swos/rgx/actions/workflows/ci.yml)
[![Crates.io](https://img.shields.io/crates/v/rgx-cli.svg)](https://crates.io/crates/rgx-cli)
[![Downloads](https://img.shields.io/crates/d/rgx-cli.svg)](https://crates.io/crates/rgx-cli)
[![License](https://img.shields.io/crates/l/rgx.svg)](LICENSE-MIT)

Test and debug regular expressions without leaving your terminal. Written in Rust.

![demo](https://raw.githubusercontent.com/brevity1swos/rgx/main/assets/demo.gif?v=6)

*Press F1 in the app for a multi-page cheat sheet with keyboard shortcuts, regex syntax, and engine-specific features.*

</div>

---

## Who is this for?

rgx is useful if you:

- **Work on remote servers** where opening a browser isn't practical — SSH sessions, containers, air-gapped environments
- **Want to pipe regex results** into other commands (`echo "log" | rgx -p '\d+' | sort`) — regex101 can't do this
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
- **Non-interactive batch mode** — `rgx -p -t "input" 'pattern'` prints matches to stdout and exits
- **JSON output** — `--json` outputs structured match data with positions and capture groups
- **Colored output** — `--color auto|always|never` highlights matches like `grep --color`
- **Pipeline composability** — pipe in, filter, pipe out: `cat log | rgx -p '\d+' | sort -n`
- **regex101 URL export** — Ctrl+U generates a regex101.com URL and copies to clipboard
- **Project workspaces** — `-w project.toml` saves/loads regex state to a file — track in git
- **Shell completions** — `--completions bash|zsh|fish` for tab completion
- **Vim mode** — optional modal editing (`--vim` or `vim_mode = true`) with Normal/Insert modes, h/j/k/l navigation, w/b/e word motions, dd/cc/x editing, and all global shortcuts preserved
- **Recipe library** — built-in common patterns (email, URL, IP, semver, etc.) — Ctrl+R to browse and load
- **Code generation** — Ctrl+G generates ready-to-use code in Rust, Python, JavaScript, Go, Java, C#, PHP, or Ruby — copies to clipboard
- **Auto engine selection** — automatically upgrades to fancy-regex or PCRE2 when your pattern uses lookahead, backreferences, or recursion
- **Test suite mode** — `rgx --test file.toml` validates regex against should-match/should-not-match assertions — CI-friendly exit codes
- **Alternating match colors** — adjacent matches use distinct background colors for visual clarity
- **Step-through debugger** — Ctrl+D opens a full-screen debugger that traces PCRE2 execution step by step, highlights the current pattern token and input position, shows backtracking, and includes a heatmap mode to reveal catastrophic backtracking at a glance
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

### Arch Linux (AUR)

```bash
yay -S rgx-cli        # or rgx-cli-bin for prebuilt
```

<details>
<summary>More installation methods</summary>

### From source

```bash
git clone https://github.com/brevity1swos/rgx.git
cd rgx
cargo install --path .
```

### With PCRE2 engine (requires libpcre2-dev)

```bash
cargo install rgx-cli --features pcre2-engine
```

> **Security note:** PCRE2 **10.45** is affected by [CVE-2025-58050](https://nvd.nist.gov/vuln/detail/CVE-2025-58050) — a heap-buffer-overflow reachable via patterns that use scan-substring `(*scs:)` verbs combined with backreferences. If you build rgx with `pcre2-engine` and link it against PCRE2 10.45, rgx will display a warning in the status bar. Upgrade your system's PCRE2 package to **>= 10.46** to resolve.

</details>

## Usage

```bash
# Interactive mode
rgx

# Vim mode (modal editing)
rgx --vim

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

# Non-interactive batch mode (--print / -p)
rgx -p -t "hello 42 world 99" '\d+'    # prints: 42\n99
echo "log line 404" | rgx -p '\d+'     # prints: 404

# Count matches only (--count / -c)
echo "a1 b2 c3" | rgx -p -c '\d+'     # prints: 3

# Extract a specific capture group (--group / -g)
echo "user@host" | rgx -p -g 1 '(\w+)@(\w+)'  # prints: user

# Batch replacement
rgx -p -t "user@host" -r '$2=$1' '(\w+)@(\w+)'   # prints: host=user

# JSON output with match positions and capture groups
echo "error at line 42" | rgx -p '(\d+)' --json

# Colored output (like grep --color)
cat server.log | rgx -p 'ERROR: (.*)' --color always

# Pipeline composability
cat server.log | rgx -p 'ERROR: (.*)' | sort | uniq -c

# Project workspace — save/load regex state to a file
rgx -w ~/project/regex/parse_urls.toml

# Capture final pattern after interactive editing
PATTERN=$(rgx -P)

# Test suite mode — validate regex assertions in CI
rgx --test tests/url_patterns.toml tests/email_patterns.toml

# Generate shell completions
rgx --completions bash > ~/.local/share/bash-completion/completions/rgx

# Exit codes: 0 = match found, 1 = no match, 2 = error
rgx -p -t "test" '\d+' || echo "no digits found"
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
| `Ctrl+R` | Open regex recipe library |
| `Ctrl+W` | Toggle whitespace visualization |
| `Ctrl+O` | Output results to stdout and quit |
| `Ctrl+S` | Save workspace |
| `Ctrl+G` | Generate code in 8 languages (copies to clipboard) |
| `Ctrl+U` | Copy regex101.com URL to clipboard |
| `Ctrl+D` | Step-through regex debugger (PCRE2) |
| `Ctrl+B` | Benchmark pattern across all engines |
| `Ctrl+Left/Right` | Move cursor by word |
| `Alt+Up/Down` | Browse pattern history |
| `Alt+i/m/s/u/x` | Toggle flags (case, multiline, dotall, unicode, extended) |
| `F1` | Show help (Left/Right to page through) |
| `Mouse click` | Focus panel and position cursor |
| `Mouse scroll` | Scroll panel under cursor |
| `Esc` | Quit (or Normal mode in vim) |

### Vim Mode (`--vim`)

| Key | Mode | Action |
|-----|------|--------|
| `i` / `a` / `I` / `A` | Normal | Enter Insert mode (at cursor / after / line start / line end) |
| `o` / `O` | Normal | Open line below / above and enter Insert mode |
| `Esc` | Insert | Return to Normal mode |
| `h` / `j` / `k` / `l` | Normal | Left / down / up / right |
| `w` / `b` / `e` | Normal | Word forward / backward / end |
| `0` / `$` / `^` | Normal | Line start / end / first non-blank |
| `gg` / `G` | Normal | First line / last line |
| `x` | Normal | Delete character under cursor |
| `dd` | Normal | Delete line |
| `cc` | Normal | Clear line and enter Insert mode |
| `u` | Normal | Undo |
| `p` | Normal | Paste from clipboard |
| `Esc` | Normal | Quit |

All global shortcuts (`Ctrl+*`, `Alt+*`, `F1`, `Tab`) work in both modes.

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
| Built-in recipe library | Yes | No | No |
| Vim keybindings | Yes | No | No |
| Non-interactive batch mode | Yes | No | No |
| JSON output | Yes | No | No |
| Colored batch output | Yes | No | No |
| regex101 URL export | Yes | No | No |
| Code generation | Yes (8 langs) | No | No |
| Auto engine selection | Yes | No | No |
| Test suite mode | Yes | No | No |
| Step-through debugger | Yes (PCRE2) | No | No |
| Shell completions | Yes | No | No |

### vs. regex101.com

regex101.com has 8 engines, shareable permalinks, and a community pattern library. rgx doesn't try to replace it — but it now has its own step-through debugger with backtracking visualization and heatmap mode. Where rgx is useful instead:

- **Offline/remote work** — no browser or internet needed
- **Pipeline integration** — `echo data | rgx -p 'pattern' | next-command` — non-interactive batch mode with proper exit codes
- **Code generation** — Ctrl+G generates code for Rust, Python, JavaScript, Go, Java, C#, PHP, Ruby
- **Engine-specific testing** — test against Rust's `regex` crate directly (regex101 doesn't have this engine)
- **Test suite mode** — `rgx --test file.toml` for CI-integrated regex validation
- **Workspace save/restore** — save your session and pick up later (`-w project.toml`)
- **Step-through debugger** — Ctrl+D traces PCRE2 execution with backtracking visualization and heatmap — no browser needed
- **Bridge to regex101** — Ctrl+U exports your current state as a regex101.com URL for sharing

## Test Suite Mode

Validate regex patterns against assertions in CI pipelines:

```toml
# tests/urls.toml
pattern = "https?://[^\\s]+"
engine = "rust"

[[tests]]
input = "visit https://example.com today"
should_match = true

[[tests]]
input = "no url here"
should_match = false
```

```bash
rgx --test tests/urls.toml tests/emails.toml
# Exit code: 0 = all pass, 1 = failures, 2 = error
```

## Configuration

rgx looks for a config file at `~/.config/rgx/config.toml`:

```toml
default_engine = "rust"  # "rust", "fancy", or "pcre2"
vim_mode = false          # enable vim-style modal editing
```

## Editor & Terminal Integration

### VS Code

Install from the [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=brevity1swos.rgx), or search "rgx" in the Extensions panel.

**Commands** (open via `Ctrl+Shift+P` / `Cmd+Shift+P`):
- `rgx: Open` — launch rgx in the integrated terminal
- `rgx: Open with Selection` — pass selected text as test input
- `rgx: Open with Pattern` — pass selected text as the regex pattern

### Neovim

Using [lazy.nvim](https://github.com/folke/lazy.nvim):

```lua
{ dir = "path/to/rgx/plugin/nvim" }
```

Or copy `plugin/nvim/lua/rgx/init.lua` to your Neovim config:

```lua
require("rgx").setup()
```

**Commands:**
- `:Rgx` — open rgx in a floating terminal
- `:Rgx pattern` — open with a pre-filled pattern
- Visual select + `:'<,'>Rgx` — pass selection as test input

### Zed

Copy `plugin/zed/tasks.json` to your project's `.zed/tasks.json` (or merge into an existing one).

Run tasks via `Cmd+Shift+P` → "task: spawn" → select an rgx task.

### tmux

```bash
# Open rgx in a tmux popup
tmux display-popup -E -w 80% -h 80% rgx

# Open with a pattern
tmux display-popup -E -w 80% -h 80% -- rgx '\d+'
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for details.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT License ([LICENSE-MIT](LICENSE-MIT))

at your option.
