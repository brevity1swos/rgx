# rgx - Terminal Regex Tester

Launch the [rgx](https://github.com/brevity1swos/rgx) regex debugger directly in the VS Code integrated terminal.

## Requirements

rgx must be installed and available on your PATH.

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
| `rgx: Open with Selection` | Launch rgx with the current selection passed as `--text` |
| `rgx: Open with Pattern` | Launch rgx with the current selection as the initial pattern argument |

Access commands via the Command Palette (`Ctrl+Shift+P` / `Cmd+Shift+P`) and search for `rgx`.

## Settings

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `rgx.path` | string | `"rgx"` | Path to the rgx binary. Override if rgx is not on your PATH. |
| `rgx.defaultEngine` | string | _(none)_ | Default regex engine: `rust`, `fancy`, or `pcre2`. |

## Usage

1. Open a file in VS Code.
2. Optionally select text (a test string or a regex pattern).
3. Open the Command Palette and run one of the `rgx:` commands.
4. rgx launches in the integrated terminal with your selection pre-loaded.
