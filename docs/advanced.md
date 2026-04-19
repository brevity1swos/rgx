# Advanced Topics

## Test suite mode

Validate regex patterns against assertions in CI pipelines:

```toml
# tests/urls.toml
pattern = "https?://[^\\s]+"
engine = "rust"

[[tests]]
input = "visit https://example.com today"
should_match = true

[[tests]]
input = "no url here"
should_match = false
```

```bash
rgx --test tests/urls.toml tests/emails.toml
# Exit: 0 = all pass, 1 = failures, 2 = error
```

## Configuration

rgx looks for a config file at `~/.config/rgx/config.toml`:

```toml
default_engine = "rust"  # "rust", "fancy", or "pcre2"
vim_mode = false         # enable vim-style modal editing
```

## Engines

| Engine | Features | Dependencies |
|--------|----------|--------------|
| **Rust `regex`** (default) | Fast, linear time, Unicode | Pure Rust |
| **fancy-regex** | + lookaround, backreferences | Pure Rust |
| **PCRE2** | + possessive quantifiers, recursion, conditionals | Requires `libpcre2` |

Auto engine selection: rgx upgrades to `fancy-regex` or `PCRE2` automatically
when your pattern uses a feature the default engine doesn't support
(lookahead, backreferences, recursion). Override with `--engine` or the
config file.

### Building with PCRE2

```bash
cargo install rgx-cli --features pcre2-engine
```

> **Security note:** PCRE2 **10.45** is affected by
> [CVE-2025-58050](https://nvd.nist.gov/vuln/detail/CVE-2025-58050) —
> a heap-buffer-overflow reachable via patterns that combine scan-substring
> `(*scs:)` verbs with backreferences. When rgx links against PCRE2 10.45
> it surfaces a warning in the status bar. Upgrade the system `libpcre2`
> package to `>= 10.46` to resolve.

## Comparison vs. terminal alternatives

| Feature | rgx | regex-tui | rexi |
|---------|:---:|:---------:|:----:|
| Real-time matching | Yes | Yes | Yes |
| Multiple engines | 3 | 2 | 1 |
| Capture group highlighting | Yes | No | No |
| Plain-English explanations | Yes | No | No |
| Replace/substitution | Yes | No | No |
| Match clipboard copy | Yes | No | No |
| Undo/redo | Yes | No | No |
| Whitespace visualization | Yes | Yes | No |
| Mouse support | Yes | No | No |
| Regex flags toggle | Yes | Yes | No |
| Stdin pipe support | Yes | Yes | Yes |
| Built-in recipe library | Yes | No | No |
| Vim keybindings | Yes | No | No |
| Non-interactive batch mode | Yes | No | No |
| JSON output | Yes | No | No |
| Colored batch output | Yes | No | No |
| regex101 URL export | Yes | No | No |
| Code generation | Yes (8 langs) | No | No |
| Auto engine selection | Yes | No | No |
| Test suite mode | Yes | No | No |
| Step-through debugger | Yes (PCRE2) | No | No |
| Shell completions | Yes | No | No |
| Generate regex from examples | Yes (grex) | No | No |
| Live filter mode (stream/grep-like) | Yes | No | No |
| JSONL field extraction | Yes | No | No |

## vs. regex101.com

regex101.com has 8 engines, shareable permalinks, and a community pattern
library. rgx doesn't try to replace it — but:

- **Offline / remote work** — no browser or internet needed
- **Pipeline integration** — `echo data | rgx -p 'pattern' | next-command` with proper exit codes
- **Code generation** — Ctrl+G generates code for Rust, Python, JavaScript, Go, Java, C#, PHP, Ruby
- **Engine-specific testing** — runs against Rust's `regex` crate directly (regex101 doesn't have this engine)
- **Test suite mode** — CI-integrated regex validation via TOML
- **Workspace save/restore** — `-w project.toml` tracks state in git
- **Step-through debugger** — Ctrl+D traces PCRE2 execution with backtracking visualization and a heatmap view
- **Bridge to regex101** — Ctrl+U exports your current state as a regex101.com URL
