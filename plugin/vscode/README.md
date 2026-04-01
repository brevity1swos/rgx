# rgx - Terminal Regex Debugger

Test, debug, and visualize regex patterns without leaving VS Code. [rgx](https://github.com/brevity1swos/rgx) runs in the integrated terminal with real-time matching, capture group highlighting, and plain-English explanations.

## Features

- **Real-time matching** — see matches update as you type
- **Capture group highlighting** — colored groups with numbered labels
- **Plain-English explanations** — understand any pattern at a glance
- **Multi-engine support** — switch between Rust regex, fancy-regex (lookaround/backrefs), and PCRE2
- **Replace mode** — test substitutions with live preview
- **Pattern history** — browse and recall previous patterns
- **Vim mode** — optional vim keybindings

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

## Commands

| Command | Description |
|---------|-------------|
| `rgx: Open` | Launch rgx in the integrated terminal |
| `rgx: Open with Selection` | Launch rgx with the current selection as test text |
| `rgx: Open with Pattern` | Launch rgx with the current selection as the regex pattern |

Access commands via the Command Palette (`Ctrl+Shift+P` / `Cmd+Shift+P`) and search for `rgx`.

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
