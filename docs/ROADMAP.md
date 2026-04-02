# rgx Roadmap

## Recently Shipped

- **Code Generation (Ctrl+G)** — Generate code in 8 languages (Rust, Python, JS, Go, Java, C#, PHP, Ruby). Closes the biggest feature gap vs regex101.
- **Test Suite Mode (--test)** — Validate regex against should-match/should-not-match assertions in TOML files. CI-friendly exit codes.
- **Alternating Match Colors** — Adjacent matches use distinct background colors for visual clarity.
- **Auto Engine Selection** — Detects lookahead, backreferences, recursion and auto-upgrades to the simplest engine that supports them.

## Next Up

### Step-through regex debugger
Visualize how the regex engine processes the pattern character by character. Show backtracking, match attempts, and where the engine gives up.

**Why**: regex101's most requested feature. No terminal tool does this. Would be a major differentiator.

### Theme customization
Support `[theme]` section in `config.toml` for custom colors.

**Why**: Accessibility. Some users need high-contrast or custom color schemes.

### Import from regex101 URL
Parse a regex101.com URL and load the pattern, test string, and flags.

**Why**: Complements the existing Ctrl+U export. Enables round-tripping between rgx and regex101.

## Future Considerations

| Feature | Impact | Effort |
|---------|--------|--------|
| More engines (JS, Python `re`) | Medium | High |
| User-saved pattern library | Medium | Medium |
| Visual railroad diagrams (ASCII) | High | Very High |
| Regex generation from examples (grex integration) | Medium | Medium |

## Not Planned

- AI/LLM integration
- Web version
- In-place file modification (sd/sed territory)
- Community pattern sharing platform
