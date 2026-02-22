# VHS demo tape for rgx
# https://github.com/charmbracelet/vhs
# Run: vhs assets/demo.tape

Output assets/demo.gif

Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set TypingSpeed 50ms

# Launch rgx
Type "rgx"
Enter
Sleep 1s

# Type a regex pattern
Type "(\w+)@(\w+)\.(\w+)"
Sleep 500ms

# Switch to test string
Tab
Sleep 300ms

# Type test strings
Type "user@example.com admin@test.org"
Sleep 2s

# Switch engine
Ctrl+e
Sleep 1s

# Switch back
Ctrl+e
Sleep 1s

# Toggle case insensitive
Alt+i
Sleep 1s

# Show help
Type "?"
Sleep 2s

# Close help
Type " "
Sleep 1s

# Quit
Escape
