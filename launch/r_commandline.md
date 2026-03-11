# r/commandline Post (v2 — March 2026)

## Title

rgx — test regex patterns from the terminal with live matching, batch mode, and 3 engines

## Body

I built **rgx** for testing and debugging regex patterns without leaving the terminal.

**GitHub:** https://github.com/brevity1swos/rgx

**Install:**
```
cargo install rgx-cli
brew install brevity1swos/tap/rgx
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/brevity1swos/rgx/releases/latest/download/rgx-installer.sh | sh
```

### Two modes

**Interactive TUI** — type a pattern and test string, see matches highlighted in real time:
- 3 regex engines (Rust regex, fancy-regex, PCRE2) — Ctrl+E to compare behavior
- Capture group highlighting with named groups
- Replace/substitution with live preview
- Plain-English pattern explanations
- Built-in regex recipe library (Ctrl+R) — common patterns for emails, IPs, dates, etc.

**Batch mode** — fits into pipelines without entering the TUI:
```
# Extract matches to stdout
echo "error at line 42" | rgx -p '\d+'
# 42

# Chain with other commands
cat server.log | rgx -p 'ERROR: (.*)' | sort | uniq -c

# Capture a pattern interactively, use it in a script
PATTERN=$(rgx -P)
grep -P "$PATTERN" *.log
```

Exit codes: 0 = match, 1 = no match, 2 = error — works with `&&`, `||`, `set -e`.

### Who this is for

- Working over SSH or in containers where opening a browser isn't practical
- Want regex results piped into other commands
- Need to test against a specific engine (PCRE2 behaves differently from Rust's `regex` crate)

If regex101.com works for your workflow, it's the more feature-rich tool. rgx fills a gap for terminal-centric work.

Cross-platform (Linux, macOS, Windows). Single binary.

Feedback welcome — what would make this more useful for your workflow?
