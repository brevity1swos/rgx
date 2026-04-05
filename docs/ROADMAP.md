# rgx Roadmap

## Recently Shipped

- **Step-Through Debugger (Ctrl+D)** — PCRE2 callout-based step-through debugger with dual-cursor visualization, backtrack markers, heatmap mode, and debug-from-selected-match. No other terminal regex tool has this.
- **Code Generation (Ctrl+G)** — Generate code in 8 languages (Rust, Python, JS, Go, Java, C#, PHP, Ruby). Closes the biggest feature gap vs regex101.
- **Test Suite Mode (--test)** — Validate regex against should-match/should-not-match assertions in TOML files. CI-friendly exit codes.
- **Alternating Match Colors** — Adjacent matches use distinct background colors for visual clarity.
- **Auto Engine Selection** — Detects lookahead, backreferences, recursion and auto-upgrades to the simplest engine that supports them.

## Next Up

### Interactive grep mode (`--grep`)
Live-filter files with regex as you type — an interactive ripgrep with rgx's pattern highlighting, capture groups, engine switching, and debugger.

```bash
rgx --grep src/          # interactively search files
rgx --grep server.log    # live-filter log file
cat data.csv | rgx --grep  # filter stdin interactively
```

**Why**: Turns rgx from occasional-use to daily-use. ripgrep (50k+ stars) has no interactive mode. fzf has no regex power. This fills the gap. Highest adoption ceiling of any planned feature.

### Regex generation from examples (grex integration)
"I have strings, give me the pattern." Interactive UI to input example strings and generate a matching regex, powered by the `grex` crate.

**Why**: Flips the workflow — appeals to developers who *avoid* regex because it's hard. Broadens the audience beyond regex experts. Low effort since grex is an existing Rust crate.

### Multi-file search/replace with preview
Interactive preview of replacements across a directory before applying. Like a TUI `sed -i` with safety.

```bash
rgx --replace src/ 'oldPattern' 'newTemplate'
```

**Why**: Real pain point — people fear `sed -i` because it's destructive. Interactive preview with confirmation makes regex replace safe. Compounds with the debugger: debug the pattern, then apply it across files.

## Future Considerations

| Feature | Impact | Effort |
|---------|--------|--------|
| Theme customization | Medium | Low |
| Import from regex101 URL | Low | Low |
| More engines (JS, Python `re`) | Medium | High |
| User-saved pattern library | Medium | Medium |

## Not Planned

- AI/LLM integration
- Web version
- Community pattern sharing platform
