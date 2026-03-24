# Editor & Terminal Integrations Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add VS Code extension, Zed tasks, tmux recipes, and README integration docs so users can launch rgx from their editor/terminal multiplexer.

**Architecture:** VS Code extension is a minimal TypeScript project (~80 lines) that registers 3 commands to open rgx in the integrated terminal. Zed integration uses a static tasks.json. tmux is README-only. Neovim plugin already exists — just needs documentation.

**Tech Stack:** TypeScript (VS Code extension API), JSON (Zed tasks), Markdown (docs)

**Spec:** `docs/superpowers/specs/2026-03-24-editor-plugins-design.md`

---

## File Structure

| File | Action | Responsibility |
|------|--------|----------------|
| `plugin/vscode/package.json` | Create | VS Code extension manifest: name, commands, config, activation, engines |
| `plugin/vscode/src/extension.ts` | Create | Register 3 commands, build shell args, create terminal |
| `plugin/vscode/tsconfig.json` | Create | TypeScript config targeting ES2020/commonjs |
| `plugin/vscode/.vscodeignore` | Create | Exclude src/, node_modules from .vsix |
| `plugin/vscode/README.md` | Create | Marketplace listing description |
| `plugin/zed/tasks.json` | Create | 3 Zed task definitions for rgx |
| `.github/workflows/vscode-ext.yml` | Create | CI: build + publish VS Code extension on tag |
| `README.md` | Modify | Add "Editor & Terminal Integration" section after "Configuration" |

---

### Task 1: VS Code Extension — package.json

**Files:**
- Create: `plugin/vscode/package.json`

- [ ] **Step 1: Create extension manifest**

```json
{
  "name": "rgx",
  "displayName": "rgx - Terminal Regex Tester",
  "description": "Launch the rgx regex debugger in the integrated terminal",
  "version": "0.1.0",
  "publisher": "brevity1swos",
  "license": "MIT",
  "repository": {
    "type": "git",
    "url": "https://github.com/brevity1swos/rgx"
  },
  "engines": {
    "vscode": "^1.85.0"
  },
  "categories": ["Other"],
  "keywords": ["regex", "tui", "terminal", "debugger"],
  "activationEvents": [],
  "main": "./out/extension.js",
  "contributes": {
    "commands": [
      {
        "command": "rgx.open",
        "title": "rgx: Open"
      },
      {
        "command": "rgx.openWithSelection",
        "title": "rgx: Open with Selection"
      },
      {
        "command": "rgx.openWithPattern",
        "title": "rgx: Open with Pattern"
      }
    ],
    "configuration": {
      "title": "rgx",
      "properties": {
        "rgx.path": {
          "type": "string",
          "default": "rgx",
          "description": "Path to the rgx binary"
        },
        "rgx.defaultEngine": {
          "type": "string",
          "enum": ["rust", "fancy", "pcre2"],
          "description": "Default regex engine to use"
        }
      }
    }
  },
  "scripts": {
    "compile": "tsc -p ./",
    "package": "vsce package",
    "publish": "vsce publish"
  },
  "devDependencies": {
    "@types/vscode": "^1.85.0",
    "typescript": "^5.3.0",
    "@vscode/vsce": "^3.0.0"
  }
}
```

- [ ] **Step 2: Create tsconfig.json**

Create `plugin/vscode/tsconfig.json`:

```json
{
  "compilerOptions": {
    "module": "commonjs",
    "target": "ES2020",
    "outDir": "out",
    "lib": ["ES2020"],
    "sourceMap": true,
    "rootDir": "src",
    "strict": true
  },
  "exclude": ["node_modules"]
}
```

- [ ] **Step 3: Commit**

```bash
git add plugin/vscode/package.json plugin/vscode/tsconfig.json
git commit -m "feat(vscode): add extension manifest and tsconfig"
```

---

### Task 2: VS Code Extension — extension.ts

**Files:**
- Create: `plugin/vscode/src/extension.ts`

- [ ] **Step 1: Write the extension source**

```typescript
import * as vscode from "vscode";

function getConfig() {
  const config = vscode.workspace.getConfiguration("rgx");
  return {
    path: config.get<string>("path", "rgx"),
    defaultEngine: config.get<string>("defaultEngine", ""),
  };
}

function getSelectedText(): string {
  const editor = vscode.window.activeTextEditor;
  if (!editor) return "";
  const selection = editor.selection;
  return editor.document.getText(selection);
}

function shellEscape(arg: string): string {
  if (process.platform === "win32") {
    // PowerShell escaping: wrap in single quotes, double internal single quotes
    return `'${arg.replace(/'/g, "''")}'`;
  }
  // Unix: wrap in single quotes, escape internal single quotes
  return `'${arg.replace(/'/g, "'\\''")}'`;
}

function buildCommand(args: string[]): string {
  const { path, defaultEngine } = getConfig();
  const parts = [path];
  if (defaultEngine) {
    parts.push("--engine", defaultEngine);
  }
  parts.push(...args);
  return parts.join(" ");
}

function runInTerminal(cmd: string) {
  const terminal = vscode.window.createTerminal({ name: "rgx" });
  terminal.sendText(cmd);
  terminal.show();
}

export function activate(context: vscode.ExtensionContext) {
  context.subscriptions.push(
    vscode.commands.registerCommand("rgx.open", () => {
      runInTerminal(buildCommand([]));
    }),

    vscode.commands.registerCommand("rgx.openWithSelection", () => {
      const text = getSelectedText();
      const args = text ? ["--text", shellEscape(text)] : [];
      runInTerminal(buildCommand(args));
    }),

    vscode.commands.registerCommand("rgx.openWithPattern", () => {
      const text = getSelectedText();
      const args = text ? [shellEscape(text)] : [];
      runInTerminal(buildCommand(args));
    })
  );
}

export function deactivate() {}
```

- [ ] **Step 2: Verify it compiles**

```bash
cd plugin/vscode && npm install && npm run compile
```

Expected: `out/extension.js` is created with no errors.

- [ ] **Step 3: Commit**

```bash
git add plugin/vscode/src/extension.ts
git commit -m "feat(vscode): implement 3 commands for terminal integration"
```

---

### Task 3: VS Code Extension — Packaging Files

**Files:**
- Create: `plugin/vscode/.vscodeignore`
- Create: `plugin/vscode/README.md`

- [ ] **Step 1: Create .vscodeignore**

```
src/
node_modules/
tsconfig.json
.vscode/
```

- [ ] **Step 2: Create marketplace README**

```markdown
# rgx - Terminal Regex Tester

Launch [rgx](https://github.com/brevity1swos/rgx) in the VS Code integrated terminal.

## Requirements

rgx must be installed and available on your PATH:

```bash
cargo install rgx-cli
# or
brew install brevity1swos/tap/rgx
```

## Commands

| Command | Description |
|---------|-------------|
| `rgx: Open` | Open rgx in a new terminal |
| `rgx: Open with Selection` | Open rgx with selected text as test input |
| `rgx: Open with Pattern` | Open rgx with selected text as the regex pattern |

## Settings

| Setting | Default | Description |
|---------|---------|-------------|
| `rgx.path` | `rgx` | Path to the rgx binary |
| `rgx.defaultEngine` | (none) | Default engine: `rust`, `fancy`, or `pcre2` |
```

- [ ] **Step 3: Verify packaging**

```bash
cd plugin/vscode && npx vsce package --no-dependencies
```

Expected: `rgx-0.1.0.vsix` is created.

- [ ] **Step 4: Commit**

```bash
git add plugin/vscode/.vscodeignore plugin/vscode/README.md
git commit -m "feat(vscode): add packaging files and marketplace README"
```

---

### Task 4: Zed Tasks

**Files:**
- Create: `plugin/zed/tasks.json`

- [ ] **Step 1: Create Zed tasks file**

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

- [ ] **Step 2: Commit**

```bash
git add plugin/zed/tasks.json
git commit -m "feat(zed): add task definitions for rgx integration"
```

---

### Task 5: README — Editor & Terminal Integration Section

**Files:**
- Modify: `README.md` (insert after the "Configuration" section, ~line 250)

- [ ] **Step 1: Add the integration section**

Insert the following after the "Configuration" section in README.md:

```markdown
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
```

- [ ] **Step 2: Verify README renders correctly**

Skim the README to confirm the new section is properly placed and formatted.

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit -m "docs: add Editor & Terminal Integration section to README"
```

---

### Task 6: VS Code Extension CI Workflow

**Files:**
- Create: `.github/workflows/vscode-ext.yml`

- [ ] **Step 1: Create the workflow**

```yaml
name: Publish VS Code Extension

on:
  push:
    tags:
      - 'vscode-v*'

jobs:
  publish:
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: plugin/vscode
    steps:
      - uses: actions/checkout@v6

      - uses: actions/setup-node@v4
        with:
          node-version: 20

      - run: npm install

      - run: npm run compile

      - name: Package extension
        run: npx vsce package --no-dependencies

      - name: Publish to Marketplace
        run: npx vsce publish --no-dependencies
        env:
          VSCE_PAT: ${{ secrets.VSCE_PAT }}

      - name: Upload .vsix artifact
        uses: actions/upload-artifact@v4
        with:
          name: rgx-vscode
          path: plugin/vscode/*.vsix
```

- [ ] **Step 2: Commit**

```bash
git add .github/workflows/vscode-ext.yml
git commit -m "ci: add VS Code extension publish workflow"
```

---

### Task 7: Final Verification and Push

- [ ] **Step 1: Verify VS Code extension compiles**

```bash
cd plugin/vscode && npm install && npm run compile
```

Expected: No errors, `out/extension.js` exists.

- [ ] **Step 2: Verify all existing tests still pass**

```bash
cargo test --no-default-features
```

Expected: All tests pass (no Rust changes were made).

- [ ] **Step 3: Push all commits**

```bash
git push origin main
```

- [ ] **Step 4: Note for VS Code Marketplace publishing**

Before the first publish, the user must:
1. Create a publisher at https://marketplace.visualstudio.com/manage
2. Generate a Personal Access Token (PAT) with Marketplace publish scope
3. Add the PAT as `VSCE_PAT` secret in the GitHub repo settings
4. Tag and push: `git tag vscode-v0.1.0 && git push origin vscode-v0.1.0`
