# r/rust Post

## Title

rgx — a terminal regex debugger with 3 engines, syntax highlighting, and plain-English explanations (built with ratatui)

## Body

I've been working on **rgx**, a terminal regex debugger that tries to bring the regex101.com experience to your terminal.

**GitHub:** https://github.com/brevity1swos/rgx

**Install:** `cargo install rgx-cli` or `brew install brevity1swos/tap/rgx`

### What it does

- **Real-time matching** — matches update on every keystroke
- **3 regex engines** — Rust `regex` (default), `fancy-regex` (lookaround/backrefs), PCRE2 (full features). Switch with Ctrl+E to compare behavior.
- **Capture group highlighting** — distinct Catppuccin colors per group, supports named captures
- **Plain-English explanations** — walks the `regex-syntax` HIR AST to generate human-readable breakdowns
- **Pattern syntax highlighting** — groups, quantifiers, character classes, anchors, escapes all get distinct colors
- **Replace/substitution mode** — live preview with `$1`, `${name}`, `$0`/`$&` syntax
- **Whitespace visualization** — toggle with Ctrl+W to show spaces/newlines/tabs
- **Undo/redo + pattern history** — Ctrl+Z, Alt+Up/Down to browse past patterns
- **Mouse support** — click panels, scroll to navigate
- **Multi-page cheat sheet** — F1 with engine-specific syntax reference

### Technical stack

Built with `ratatui` + `crossterm` + `tokio` for the TUI, `regex-syntax` for AST walking, `arboard` for clipboard. Compiles to a single binary. PCRE2 is behind a feature flag so you can build with zero C dependencies (`cargo install rgx-cli --no-default-features`).

The engine abstraction was fun to design — `RegexEngine` trait compiles patterns, `CompiledRegex` trait finds matches. Adding a new engine is just implementing both traits and adding a variant to the `EngineKind` enum.

### Why I built it

I got tired of switching between my terminal and browser to test regexes. The multi-engine comparison is what really hooked me — being able to instantly check if a pattern behaves differently between Rust's regex crate and PCRE2 has caught subtle bugs multiple times.

Would love feedback from the ratatui community on the UI patterns and anyone who works with regexes regularly.
