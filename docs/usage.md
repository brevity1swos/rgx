# Usage

Full CLI reference for rgx — interactive, batch, and `rgx filter` modes.

## Interactive TUI

```bash
rgx                                 # empty editor
rgx --vim                           # modal editing (vim mode)
rgx '\d{3}-\d{3}-\d{4}'             # start with a pattern
echo "Call 555-123-4567" | rgx '\d+' # pipe stdin as test string
rgx --engine fancy '\w+(?=@)'       # pick an engine
rgx -i 'hello'                      # case-insensitive flag
rgx -r '$2/$1' '(\w+)@(\w+)'        # pattern + replace template
```

## Batch mode (`--print` / `-p`)

`-p` bypasses the TUI entirely. Useful in shell pipelines.

```bash
rgx -p -t "hello 42 world 99" '\d+'         # → 42\n99
echo "log line 404" | rgx -p '\d+'          # → 404
echo "a1 b2 c3" | rgx -p -c '\d+'           # → 3   (count)
echo "user@host" | rgx -p -g 1 '(\w+)@(\w+)' # → user (capture group 1)
rgx -p -t "user@host" -r '$2=$1' '(\w+)@(\w+)'  # → host=user (replace)
echo "error at 42" | rgx -p '(\d+)' --json  # structured match data
cat server.log | rgx -p 'ERROR: (.*)' --color always
cat server.log | rgx -p 'ERROR: (.*)' | sort | uniq -c
```

Exit codes: `0` = match found, `1` = no match, `2` = error.

## `rgx filter` — interactive stream filter

Reads stdin or a file, lets you refine a regex against the stream in a TUI,
and emits matching lines on Enter. Non-TTY stdout skips the TUI entirely
and behaves like `grep`, so it composes in pipelines.

```bash
# Count error lines in a log
cat /var/log/system.log | rgx filter --count 'error|fail'

# Emit only non-matching lines (like grep -v)
cat access.log | rgx filter -v '200 '

# Prefix matches with line numbers
rgx filter -f server.log -n 'Exception'
```

### JSONL field extraction (`--json`)

When the input is JSONL (one JSON object per line), the default behavior
matches the pattern against the full raw line — which often catches
timestamps, IDs, or other metadata by accident. `--json <PATH>` filters on
a specific field instead:

```bash
# Match only the `msg` field of each JSONL record, not the raw line
cat app.jsonl | rgx filter --json '.msg' '(?i)error'

# Nested text field
cat events.jsonl | rgx filter --json '.payload.text' --count '^boom'

# Keys that aren't bare identifiers — hyphens, dots, spaces, unicode —
# use bracketed string form:
cat logs.jsonl | rgx filter --json '["user-id"]' '^u[0-9]+'
cat kr.jsonl  | rgx filter --json '.["日本語"]' '^언어'
```

Path language: `.field` for object keys, `[N]` for array indices,
`["..."]` for arbitrary string keys (supports `\"` and `\\` escapes).
Nesting: `.a.b[0].c`. Non-string values, missing paths, and malformed
JSON lines are skipped silently. Raw JSON lines are still what gets
emitted when a match hits — downstream tools receive the full record.

### Filtering diffs

```bash
# Every diff line that adds a console.log call
git diff | rgx filter '^\+.*console\.log'

# Count TODO markers added in the working tree
git diff HEAD | rgx filter --count '^\+.*TODO'
```

Once a pattern matches what you want interactively, paste it into any
policy or CI check.

### Filter mode caps

- `--max-lines N` — drop input after N lines (default 100 000, `0` disables). Prevents runaway pipes from OOMing.
- Per-line byte cap — any single line over 10 MiB is truncated automatically. Protects against unterminated binary input.

## Workspaces, test suites, completions

```bash
# Save/restore interactive state (tracked in git)
rgx -w ~/project/regex/parse_urls.toml

# Capture the final pattern after interactive editing
PATTERN=$(rgx -P)

# CI-friendly regex validation (see docs/advanced.md)
rgx --test tests/url_patterns.toml tests/email_patterns.toml

# Shell completions
rgx --completions bash > ~/.local/share/bash-completion/completions/rgx
rgx --completions zsh  > ~/.zsh/completion/_rgx
rgx --completions fish > ~/.config/fish/completions/rgx.fish
```
