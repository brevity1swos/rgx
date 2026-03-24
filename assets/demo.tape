# VHS demo tape for rgx
# https://github.com/charmbracelet/vhs
# Run: PATH=$HOME/.cargo/bin:$PATH vhs assets/demo.tape
#
# NOTE: VHS does not support F1, so the cheat sheet overlay is not shown here.
# The help overlay can be opened with F1 and paged with Left/Right in the app.

Output assets/demo.gif

Set FontSize 14
Set Width 1200
Set Height 650
Set Theme "Catppuccin Mocha"
Set TypingSpeed 40ms
Set Shell "bash"

# === Part 1: Batch mode (new in v0.8.0) ===

# JSON output
Type "echo 'error at line 42, col 7' | rgx -p '(\d+)' --json"
Enter
Sleep 3s

# Colored output
Type "echo 'hello 42 world 99 foo' | rgx -p '\d+' --color always"
Enter
Sleep 3s

# === Part 2: Interactive TUI ===

# Launch rgx
Type "rgx"
Enter
Sleep 1s

# --- Pattern matching with captures ---
Type "(\w+)@(\w+)\.(\w+)"
Sleep 2s

# Switch to test string input
Tab
Sleep 300ms

# Type multi-line test string
Type "user@example.com"
Enter
Type "admin@test.org"
Enter
Type "hello world"
Sleep 2s

# --- Whitespace visualization ---
Ctrl+w
Sleep 2s

# Toggle off
Ctrl+w
Sleep 1s

# --- Replace mode ---
Tab
Sleep 300ms

Type "$1 AT $2"
Sleep 2s

# --- Undo/Redo ---
Ctrl+z
Sleep 400ms
Ctrl+z
Sleep 400ms
Ctrl+z
Sleep 400ms
Ctrl+z
Sleep 400ms

# Redo it back
Ctrl+Shift+z
Sleep 400ms
Ctrl+Shift+z
Sleep 400ms
Ctrl+Shift+z
Sleep 400ms
Ctrl+Shift+z
Sleep 1.5s

# --- Match selection + clipboard ---
Tab
Sleep 300ms

Down
Sleep 300ms
Down
Sleep 300ms
Down
Sleep 300ms
Down
Sleep 300ms

# Copy selected match
Ctrl+y
Sleep 2s

# Navigate back up
Up
Sleep 250ms
Up
Sleep 250ms
Up
Sleep 250ms
Up
Sleep 250ms

Sleep 500ms

# --- Export regex101 URL (new in v0.8.0) ---
Ctrl+u
Sleep 2.5s

# --- Engine switching ---
Ctrl+e
Sleep 1.5s

Ctrl+e
Sleep 1s

# --- Flag toggle ---
Alt+i
Sleep 1s
Alt+i
Sleep 1s

# Quit
Escape
Sleep 500ms
