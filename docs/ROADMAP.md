# rgx Roadmap

## Next Up

### Code Generation (Ctrl+G)
Generate ready-to-use code from the current pattern and flags. Select a language, copies to clipboard.

Supported languages: Rust, Python, JavaScript, Go, Java, C#, PHP, Ruby.

Maps engine flags to language-specific syntax. Template-based, no AI.

**Why**: #1 reason people use regex101 over terminal tools. Closes the biggest feature gap.

### Test Suite Mode (--test)
Expand workspaces (`-w`) to support should-match / should-not-match assertions:

```toml
pattern = "https?://[^\s]+"
[[tests]]
input = "visit https://example.com"
should_match = true
[[tests]]
input = "no url here"
should_match = false
```

Run with `rgx --test file.toml` for CI-integrated regex validation (exit code 0/1).

**Why**: Strongest external signal — workspace PR from a non-Rust developer. Positions rgx as a regex development environment, not just a debugger.

### Alternating Match Colors
Alternate highlight colors between adjacent matches for visual distinction.

**Why**: Easy win. Improves clarity for dense patterns.

### Auto Engine Selection
Detect pattern features (lookahead, backrefs, recursion) and pick the simplest engine that supports them automatically.

**Why**: Reduces friction. ripgrep does this. Pattern analysis already exists in the explainer.

## Future Considerations

| Feature | Impact | Effort |
|---------|--------|--------|
| More engines (JS, Python `re`) | Medium | High |
| Step-through regex debugger | High | Very High |
| User-saved pattern library | Medium | Medium |
| Theme customization (`[theme]` in config.toml) | Low-Medium | Easy |
| Import from regex101 URL | Low-Medium | Easy |

## Not Planned

- AI/LLM integration
- Web version
- In-place file modification (sd/sed territory)
- Community pattern sharing platform
