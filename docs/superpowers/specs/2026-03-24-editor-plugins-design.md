# Design: Editor & Terminal Integrations

## Goal

Add editor integrations so users can launch rgx from VS Code, Zed, Neovim, and tmux without leaving their workflow. Each integration passes selected text or patterns to rgx in a terminal.

## Scope

| Item | Type | Publish target |
|------|------|---------------|
| VS Code extension | TypeScript extension | VS Code Marketplace |
| Zed tasks | tasks.json config | Documented in README |
| tmux recipes | Shell one-liners | Documented in README |
| Neovim plugin | Already complete | Documented in README |

---

## VS Code Extension

### Directory

`plugin/vscode/`

### Commands

| Command ID | Title | Behavior |
|-----------|-------|----------|
| `rgx.open` | rgx: Open | Opens rgx in a new integrated terminal |
| `rgx.openWithSelection` | rgx: Open with Selection | Passes active editor selection as `--text` |
| `rgx.openWithPattern` | rgx: Open with Pattern | Passes active editor selection as the positional pattern argument |

### Configuration

| Setting | Type | Default | Description |
|---------|------|---------|-------------|
| `rgx.path` | string | `"rgx"` | Path to rgx binary |
| `rgx.defaultEngine` | enum: `rust`, `fancy`, `pcre2` | (none) | If set, adds `--engine <value>` |

### Files

```
plugin/vscode/
  package.json          # Extension manifest (name, commands, config, activation)
  src/extension.ts      # Single file: register 3 commands, build shell args, create terminal
  tsconfig.json         # TypeScript config
  .vscodeignore         # Exclude dev files from .vsix
  README.md             # Marketplace description
```

### Implementation

`extension.ts` (~80 lines):

- `activate()` registers 3 commands via `vscode.commands.registerCommand()`
- Each command:
  1. Reads `rgx.path` and `rgx.defaultEngine` from `vscode.workspace.getConfiguration("rgx")`
  2. Gets selected text from `vscode.window.activeTextEditor.selection` (for selection/pattern commands)
  3. Builds a shell command string with proper escaping
  4. Creates a terminal via `vscode.window.createTerminal({ name: "rgx" })`
  5. Sends the command via `terminal.sendText(cmd)`
  6. Shows the terminal via `terminal.show()`
- Activation event: `onCommand` only — extension loads on first command invocation (~100ms), instant after that
- Minimum VS Code version: 1.85+
- Shell escaping: use `shlex`-style quoting — single quotes on Unix, PowerShell escaping on Windows. Build args as an array and shell-escape each element before joining

### package.json Key Fields

```json
{
  "name": "rgx",
  "displayName": "rgx - Terminal Regex Tester",
  "description": "Launch rgx regex debugger in the integrated terminal",
  "publisher": "brevity1swos",
  "categories": ["Other"],
  "activationEvents": [],
  "main": "./out/extension.js",
  "contributes": {
    "commands": [
      { "command": "rgx.open", "title": "rgx: Open" },
      { "command": "rgx.openWithSelection", "title": "rgx: Open with Selection" },
      { "command": "rgx.openWithPattern", "title": "rgx: Open with Pattern" }
    ],
    "configuration": {
      "title": "rgx",
      "properties": {
        "rgx.path": { "type": "string", "default": "rgx" },
        "rgx.defaultEngine": { "type": "string", "enum": ["rust", "fancy", "pcre2"] }
      }
    }
  }
}
```

### Publishing

- Requires a Visual Studio Marketplace publisher account (`brevity1swos`)
- Build: `npm run compile && vsce package`
- Publish: `vsce publish`
- CI: GitHub Actions workflow triggered on tag push (e.g., `vscode-v0.1.0`), builds `.vsix`, publishes to marketplace

### CI Workflow

```
.github/workflows/vscode-ext.yml
  triggers: push tag 'vscode-v*'
  steps:
    - checkout
    - setup node
    - npm install (in plugin/vscode/)
    - npm run compile
    - vsce package
    - vsce publish (using VSCE_PAT secret)
    - upload .vsix as release artifact
```

---

## Zed Tasks

### Directory

`plugin/zed/`

### File

`plugin/zed/tasks.json` — users copy this to their `.zed/tasks.json` or project `.zed/tasks.json`.

### Task Definitions

```json
[
  {
    "label": "rgx: Open",
    "command": "rgx",
    "args": [],
    "tags": ["rgx"]
  },
  {
    "label": "rgx: Open with Selection",
    "command": "rgx",
    "args": ["--text", "$ZED_SELECTED_TEXT"],
    "tags": ["rgx"]
  },
  {
    "label": "rgx: Open with Pattern",
    "command": "rgx",
    "args": ["$ZED_SELECTED_TEXT"],
    "tags": ["rgx"]
  }
]
```

### Notes

- Zed task variables: verify `$ZED_SELECTED_TEXT` name against Zed docs (0.175+). If no text is selected, the variable expands to an empty string and rgx opens normally.
- Users run tasks via `cmd+shift+p` → "task: spawn" → select the rgx task. Selected text is substituted by Zed at runtime.

---

## tmux Recipes

No files — README documentation only.

### Recipes

```bash
# Open rgx in a tmux popup
tmux display-popup -E -w 80% -h 80% rgx

# Open with pattern
tmux display-popup -E -w 80% -h 80% -- rgx 'pattern'
```

---

## Neovim Plugin

Already complete at `plugin/nvim/lua/rgx/init.lua`. Features:

- `:Rgx` — open in floating terminal
- `:Rgx pattern` — pre-fill pattern
- Visual select + `:'<,'>Rgx` — pass selection as `--text`
- Configurable: `cmd`, `width`, `height`, `border`

No changes needed.

---

## README Integration

Add a new section "Editor & Terminal Integration" to README.md after the "Configuration" section. Contains subsections for each editor with install/usage instructions:

1. **VS Code** — marketplace link + extension ID
2. **Neovim** — lazy.nvim/packer snippet
3. **Zed** — copy tasks.json instructions
4. **tmux** — one-liner recipes

---

## Out of Scope

- Keybinding defaults (editors handle this natively)
- Theme/color customization in extensions
- Language server or regex validation features
- Webview/sidebar panels
- JetBrains/Sublime/Emacs plugins
