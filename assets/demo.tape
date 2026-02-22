# VHS demo tape for rgx
# https://github.com/charmbracelet/vhs
# Run: vhs assets/demo.tape

Output assets/demo.gif

Set FontSize 14
Set Width 1200
Set Height 600
Set Theme "Catppuccin Mocha"
Set TypingSpeed 50ms
Set Shell "zsh"

# Launch rgx
Type "rgx"
Enter
Sleep 1s

# Type a regex pattern — email matching
Type "(\w+)@(\w+)\.(\w+)"
Sleep 2s

# Switch to test string input
Tab
Sleep 500ms

# Type test strings with email addresses
Type "user@example.com admin@test.org"
Sleep 3s

# Switch engine to fancy-regex
Ctrl+e
Sleep 2s

# Switch engine to PCRE2
Ctrl+e
Sleep 2s

# Switch back to Rust regex
Ctrl+e
Sleep 1s

# Toggle case insensitive flag
Alt+i
Sleep 2s

# Toggle it back off
Alt+i
Sleep 1s

# Quit
Escape
Sleep 500ms
