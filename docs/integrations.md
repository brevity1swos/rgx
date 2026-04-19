# Editor & Terminal Integration

## VS Code

Install from the
[VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=brevity1swos.rgx),
or search "rgx" in the Extensions panel.

Commands (open via `Ctrl+Shift+P` / `Cmd+Shift+P`):

- `rgx: Open` — launch rgx in the integrated terminal
- `rgx: Open with Selection` — pass selected text as test input
- `rgx: Open with Pattern` — pass selected text as the regex pattern

## Neovim

Using [lazy.nvim](https://github.com/folke/lazy.nvim):

```lua
{ dir = "path/to/rgx/plugin/nvim" }
```

Or copy `plugin/nvim/lua/rgx/init.lua` to your Neovim config:

```lua
require("rgx").setup()
```

Commands:

- `:Rgx` — open rgx in a floating terminal
- `:Rgx pattern` — open with a pre-filled pattern
- Visual select + `:'<,'>Rgx` — pass selection as test input

## Zed

Copy `plugin/zed/tasks.json` to your project's `.zed/tasks.json` (or merge
into an existing one).

Run tasks via `Cmd+Shift+P` → "task: spawn" → select an rgx task.

## tmux

```bash
# Open rgx in a tmux popup
tmux display-popup -E -w 80% -h 80% rgx

# Open with a pattern
tmux display-popup -E -w 80% -h 80% -- rgx '\d+'
```
