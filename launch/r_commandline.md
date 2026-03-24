# r/commandline Post (v3 — March 2026)

## Title

rgx — terminal regex debugger with live matching, JSON output, colored grep-style results, and 3 engines

## Body

I built **rgx** for testing and debugging regex patterns without leaving the terminal. Just shipped v0.8.0 with some features I think this community will appreciate.

**GitHub:** https://github.com/brevity1swos/rgx

**Install:**
```
cargo install rgx-cli
brew install brevity1swos/tap/rgx
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/brevity1swos/rgx/releases/latest/download/rgx-installer.sh | sh
```

### What it does

**Interactive TUI** — type a pattern and test string, see matches highlighted in real time:
- 3 regex engines (Rust regex, fancy-regex, PCRE2) — Ctrl+E to compare behavior
- Capture group highlighting with named groups
- Replace/substitution with live preview
- Plain-English pattern explanations
- Built-in regex recipe library (Ctrl+R)
- Ctrl+U exports a regex101.com URL to clipboard — iterate locally, share the link
- Workspace save/load (`-w project.toml`) — track regex files in git

**Batch mode** — fits into pipelines without entering the TUI:
```bash
# Extract matches
echo "error at line 42, col 7" | rgx -p '\d+'
# 42
# 7

# JSON output with positions and capture groups
echo "error at line 42" | rgx -p '(\d+)' --print --json
# [{"match": "42", "start": 14, "end": 16, "groups": [{"group": 1, "value": "42"}]}]

# Colored output like grep --color
cat server.log | rgx -p 'ERROR: (.*)' --color always

# Count matches
echo "a1 b2 c3" | rgx -p -c '\d+'   # 3

# Capture a pattern interactively, use it in a script
PATTERN=$(rgx -P)
grep -P "$PATTERN" *.log
```

**Shell completions:**
```bash
rgx --completions bash > ~/.local/share/bash-completion/completions/rgx
rgx --completions zsh > ~/.zfunc/_rgx
rgx --completions fish > ~/.config/fish/completions/rgx.fish
```

Exit codes: 0 = match, 1 = no match, 2 = error — works with `&&`, `||`, `set -e`.

### Who this is for

- Working over SSH or in containers where opening a browser isn't practical
- Want regex results piped into other commands — especially the `--json` output for programmatic use
- Need to test against a specific engine (PCRE2 behaves differently from Rust's `regex` crate)
- Prefer staying in the terminal

If regex101.com works for your workflow, it's the more feature-rich tool. rgx fills a gap for terminal-centric work.

Cross-platform (Linux, macOS, Windows). Single binary, no runtime dependencies.

What would make this more useful for your workflow?
