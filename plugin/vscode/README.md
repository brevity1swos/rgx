# rgx - Terminal Regex Debugger

Test, debug, and visualize regex patterns without leaving VS Code. [rgx](https://github.com/brevity1swos/rgx) runs in the integrated terminal with real-time matching, capture group highlighting, and plain-English explanations.

## Features

- **Step-through debugger** — Ctrl+D visualizes regex engine state at each step, with backtracking markers and a heatmap mode (PCRE2)
- **Real-time matching** — see matches update as you type
- **Capture group highlighting** — colored groups with numbered labels, alternating match colors
- **Plain-English explanations** — understand any pattern at a glance
- **Live filter mode** — `rgx filter` streams stdin/file through a regex TUI; supports `--json` JSONL-field extraction, `--count`, `--line-number`, `--invert`, and grep-like exit codes
- **Quick Reference side panel** — F3 toggles a syntax cheat sheet on the right edge of the screen
- **Multi-engine support** — switch between Rust regex, fancy-regex (lookaround/backrefs), and PCRE2
- **Auto engine selection** — automatically upgrades the engine when your pattern needs lookahead, backreferences, or recursion
- **Code generation** — Ctrl+G generates ready-to-use code in Rust, Python, JavaScript, Go, Java, C#, PHP, or Ruby
- **Benchmark across engines** — Ctrl+B measures compile and match time for the same pattern on all available engines
- **Regex from examples** — Ctrl+X opens a grex overlay that infers a pattern from sample strings
- **Replace mode** — test substitutions with live preview
- **Test suite mode** — validate regex against assertions with `rgx --test file.toml`
- **Workspaces** — Ctrl+S saves pattern + test string + flags to a TOML file you can track in git
- **Pattern history + undo/redo** — browse and recall previous patterns; Ctrl+Z / Ctrl+Shift+Z
- **Recipe library** — built-in patterns for common tasks (email, URL, IP, etc.)
- **Vim mode** — optional vim keybindings
- **regex101 export** — Ctrl+U generates a shareable regex101.com URL

## Requirements

`rgx` must be installed and available on your PATH.

**Install via Cargo:**
```
cargo install rgx-cli
```

**Install via Homebrew:**
```
brew install brevity1swos/tap/rgx
```

**Install via AUR (Arch Linux):**
```
yay -S rgx-cli
```

## Commands

| Command | Description |
|---------|-------------|
| `rgx: Open` | Launch rgx in the integrated terminal |
| `rgx: Open with Selection` | Launch rgx with the current selection as test text |
| `rgx: Open with Pattern` | Launch rgx with the current selection as the regex pattern |
| `rgx: Filter Current File with Pattern` | Run `rgx filter` against the current file with the selection as pattern |

Access commands via the Command Palette (`Ctrl+Shift+P` / `Cmd+Shift+P`) and search for `rgx`.

## Key Shortcuts (inside rgx)

| Key | Action |
|-----|--------|
| `Ctrl+D` | Step-through regex debugger (PCRE2) |
| `Ctrl+G` | Generate code for the current pattern (8 languages) |
| `Ctrl+B` | Benchmark pattern across all engines |
| `Ctrl+X` | Generate regex from examples (grex overlay) |
| `Ctrl+R` | Open regex recipe library |
| `Ctrl+U` | Copy regex101.com URL to clipboard |
| `Ctrl+E` | Cycle regex engine |
| `Ctrl+Y` | Copy pattern (regex panel) or selected match (matches panel) |
| `Ctrl+W` | Toggle whitespace visualization |
| `Ctrl+S` | Save workspace |
| `Ctrl+O` | Output results to stdout and quit |
| `Ctrl+Z` / `Ctrl+Shift+Z` | Undo / redo |
| `F1` | Show help (Left/Right page, Up/Down scroll) |
| `F3` | Toggle Quick Reference side panel |
| `PgUp` / `PgDn` | Scroll Quick Reference side panel |

## Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `rgx.path` | string | `"rgx"` | Path to the rgx binary |
| `rgx.defaultEngine` | string | _(none)_ | Default regex engine: `rust`, `fancy`, or `pcre2` |

## Usage

1. Open a file in VS Code.
2. Optionally select text (a test string or a regex pattern).
3. Open the Command Palette and run one of the `rgx:` commands.
4. rgx launches in the integrated terminal with your selection pre-loaded.

## Links

- [GitHub](https://github.com/brevity1swos/rgx) — source, issues, and documentation
- [Changelog](https://github.com/brevity1swos/rgx/blob/main/CHANGELOG.md)
