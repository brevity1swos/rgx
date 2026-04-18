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

# === Part 1: Batch mode ===

# JSON output
Type "echo 'error at line 42, col 7' | rgx -p '(\d+)' --json"
Enter
Sleep 3s

# Colored output
Type "echo 'hello 42 world 99 foo' | rgx -p '\d+' --color always"
Enter
Sleep 3s

# === Part 2: Interactive TUI ===

# Launch rgx with test text to show alternating match colors
Type "rgx -t 'user@example.com admin@test.org hello world'"
Enter
Sleep 1s

# --- Pattern matching with captures + alternating colors ---
Type "(\w+)@(\w+)\.(\w+)"
Sleep 2s

# --- Whitespace visualization ---
Ctrl+w
Sleep 2s
Ctrl+w
Sleep 1s

# --- Replace mode ---
Tab
Sleep 300ms
Tab
Sleep 300ms

Type "$1 AT $2"
Sleep 2s

# --- Code generation (Ctrl+G) ---
Ctrl+g
Sleep 3s

# Browse languages
Down
Sleep 1s
Down
Sleep 1s
Down
Sleep 1s

# Back to Python
Up
Sleep 1s
Up
Sleep 1s

# Copy Python code
Enter
Sleep 3s

# --- Match selection + clipboard ---
Tab
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

Up
Sleep 250ms
Up
Sleep 250ms
Up
Sleep 250ms

Sleep 500ms

# --- Export regex101 URL ---
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

# === Part 2.5: Generate regex from examples (Ctrl+X / grex) ===

# Launch rgx empty
Type "rgx"
Enter
Sleep 1s

# Open grex overlay
Ctrl+x
Sleep 1.5s

# Type three example strings, one per line
Type "foo"
Enter
Type "bar"
Enter
Type "baz"
Sleep 2s

# Toggle case-insensitive flag to show it regenerates
Alt+c
Sleep 1.5s
Alt+c
Sleep 1s

# Tab accepts the generated pattern — loads it into the main regex editor
Tab
Sleep 2.5s

# Quit
Escape
Sleep 500ms

# === Part 3: Step-through debugger ===

# Launch with a pattern that causes backtracking
# Requires rgx built with: cargo install rgx-cli --features pcre2-engine
Type "rgx -t 'aaab' 'a+ab'"
Enter
Sleep 2s

# Open debugger with Ctrl+D
Ctrl+d
Sleep 2s

# Step through execution
Right
Sleep 500ms
Right
Sleep 500ms
Right
Sleep 500ms
Right
Sleep 500ms
Right
Sleep 500ms

# Jump to next backtrack
Type "f"
Sleep 1.5s

# Toggle heatmap
Type "H"
Sleep 2s

# Close debugger
Escape
Sleep 500ms

# Quit
Escape
Sleep 500ms

# === Part 4: Auto engine selection ===

# Launch with a lookahead pattern — auto-switches to fancy-regex
Type "rgx -t 'user@example.com admin@test.org' '\w+(?=@)'"
Enter
Sleep 3s

# Status bar shows "Auto-switched Rust regex → fancy-regex"

# Quit
Escape
Sleep 500ms
