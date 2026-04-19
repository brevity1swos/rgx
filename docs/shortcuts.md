# Keyboard Shortcuts

Press `F1` inside the app for an in-terminal cheat sheet.

## Main TUI

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
| `Ctrl+X` | Generate regex from examples (grex) |
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

## Vim mode (`--vim`)

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

## Filter mode (`rgx filter`)

| Key | Action |
|-----|--------|
| `Up/Down` | Browse matching lines |
| `Alt+i` | Toggle case-insensitive |
| `Alt+v` | Toggle invert match |
| `Enter` | Emit matched lines to stdout and exit (exit 0) |
| `Esc` / `Ctrl+C` | Discard and exit (exit 1) |
| Typing / Backspace | Edit the regex pattern (re-filters live) |
