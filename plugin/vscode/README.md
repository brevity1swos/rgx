# rgx - Terminal Regex Debugger

Test, debug, and visualize regex patterns without leaving VS Code. [rgx](https://github.com/brevity1swos/rgx) runs in the integrated terminal with real-time matching, capture group highlighting, and plain-English explanations.

## Features

- **Real-time matching** — see matches update as you type
- **Capture group highlighting** — colored groups with numbered labels, alternating match colors
- **Plain-English explanations** — understand any pattern at a glance
- **Multi-engine support** — switch between Rust regex, fancy-regex (lookaround/backrefs), and PCRE2
- **Auto engine selection** — automatically upgrades the engine when your pattern needs lookahead, backreferences, or recursion
- **Code generation** — Ctrl+G generates ready-to-use code in Rust, Python, JavaScript, Go, Java, C#, PHP, or Ruby
- **Replace mode** — test substitutions with live preview
- **Test suite mode** — validate regex against assertions with `rgx --test file.toml`
- **Pattern history** — browse and recall previous patterns
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

Access commands via the Command Palette (`Ctrl+Shift+P` / `Cmd+Shift+P`) and search for `rgx`.

## Key Shortcuts (inside rgx)

| Key | Action |
|-----|--------|
| `Ctrl+G` | Generate code for the current pattern (8 languages) |
| `Ctrl+R` | Open regex recipe library |
| `Ctrl+U` | Copy regex101.com URL to clipboard |
| `Ctrl+E` | Cycle regex engine |
| `Ctrl+Y` | Copy selected match to clipboard |
| `Ctrl+W` | Toggle whitespace visualization |
| `F1` | Show help |

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
