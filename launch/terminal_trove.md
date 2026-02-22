# Terminal Trove Submission

## Submission URL

https://terminaltrove.com/post/

## Details

### Tool Name
rgx

### Website URL
github.com/brevity1swos/rgx

### Tagline (~120 chars)
regex101 for the terminal — real-time matching, 3 engines, capture groups, replace mode, and plain-English explanations

### What the tool does
rgx is a terminal regex debugger that brings the regex101.com experience to the command line. It provides real-time matching that updates on every keystroke, with support for 3 regex engines (Rust regex, fancy-regex, PCRE2), capture group highlighting, and human-readable pattern explanations.

### 2-3 Standout Features
1. **3 regex engines** — switch between Rust regex, fancy-regex (lookaround/backrefs), and PCRE2 (full features) with Ctrl+E to compare behavior
2. **Plain-English explanations** — walks the regex AST to generate human-readable breakdowns of your pattern
3. **Replace/substitution mode** — live preview with $1, ${name}, $0/$& syntax

### Other Notable Features
- Pattern syntax highlighting with Catppuccin color palette
- Capture group highlighting with distinct colors per group and named group display
- Undo/redo and pattern history browsing
- Whitespace visualization toggle
- Mouse support (click to focus, scroll to navigate)
- Multi-page context-sensitive cheat sheet
- Match selection + clipboard copy
- Stdin pipe support

### Target Audience
Developers who work with regular expressions and want a fast, offline alternative to regex101.com. Particularly useful for Rust developers testing patterns across different regex engines, DevOps engineers debugging log patterns, and anyone who wants to stay in the terminal.

### Primary Programming Language
Rust

### License
MIT / Apache-2.0 (dual-licensed)

### Preview Image
Use the demo GIF from the repo: https://raw.githubusercontent.com/brevity1swos/rgx/main/assets/demo.gif

### Categories
- Development Tools
- Text Processing
- Productivity

### Installation Commands

**Cargo:**
```
cargo install rgx-cli
```

**Homebrew:**
```
brew install brevity1swos/tap/rgx
```

**Shell installer:**
```
curl --proto '=https' --tlsv1.2 -LsSf https://github.com/brevity1swos/rgx/releases/latest/download/rgx-installer.sh | sh
```

**Prebuilt binaries:**
Download from https://github.com/brevity1swos/rgx/releases/latest

### Cross-platform
Yes — Linux, macOS, Windows
