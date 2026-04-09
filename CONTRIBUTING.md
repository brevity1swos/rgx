# Contributing to rgx

Thank you for your interest in contributing to rgx!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/YOUR_USERNAME/rgx.git`
3. Create a branch: `git checkout -b my-feature`
4. Make your changes
5. Run the checks: `cargo fmt && cargo clippy --all-features && cargo test --all-features`
6. Commit using [conventional commits](https://www.conventionalcommits.org/): `git commit -m "feat: add cool feature"`
7. Push and open a PR

## Development Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build with all features (including PCRE2)
cargo build --all-features

# Build without PCRE2 (no C dependencies)
cargo build --no-default-features

# Run tests
cargo test --all-features

# Run clippy
cargo clippy --all-features -- -D warnings

# Run benchmarks
cargo bench
```

## Commit Messages

We use [Conventional Commits](https://www.conventionalcommits.org/):

- `feat:` — new feature
- `fix:` — bug fix
- `docs:` — documentation only
- `refactor:` — code change that neither fixes a bug nor adds a feature
- `test:` — adding or updating tests
- `perf:` — performance improvement
- `chore:` — maintenance tasks

## Code Style

- Run `cargo fmt` before committing
- All clippy lints must pass with `-D warnings`
- Write tests for new functionality
- Keep functions focused and small

## Architecture

- `src/engine/` — Regex engine abstraction, implementations (Rust regex, fancy-regex, PCRE2), auto-detection, replace/substitution, step-through debugger
- `src/explain/` — Plain-English regex explanation generator
- `src/ui/` — TUI rendering (ratatui widgets: pattern input, test input, replace input, match display, explanation, debugger overlay, status bar, syntax highlighting)
- `src/input/` — Keyboard input handling, text editing, vim mode
- `src/config/` — CLI arguments, persistent settings, workspace save/load
- `src/app.rs` — Central application state and action dispatch
- `src/event.rs` — Async event loop
- `src/codegen.rs` — Code generation for 8 languages
- `src/recipe.rs` — Built-in regex recipe library
- `src/ansi.rs` — ANSI color output for batch mode

## License

By contributing, you agree that your contributions will be licensed under the MIT OR Apache-2.0 license.
