# Changelog

All notable changes to this project will be documented in this file.

## [0.1.4] - 2026-02-22

### Bug Fixes

- Regenerate demo GIF with working rgx binary
Previous demo GIF was recorded before rgx was installed, showing
  a blank terminal. Regenerated with VHS using bash shell that
  inherits PATH with ~/.cargo/bin.


## [0.1.3] - 2026-02-22

### Bug Fixes

- Update crossterm to 0.29, clean up dead_code allows, add logo
- Bump crossterm from 0.28 to 0.29 to align with ratatui 0.30
  - Remove #![allow(dead_code)] from main.rs, lib.rs, and settings.rs
  - Have main.rs use the rgx library crate instead of re-declaring modules
  - Fix duplicate changelog header
  - Add SVG logo asset
  - Add PCRE2 to engine benchmarks (behind feature gate)


## [0.1.2] - 2026-02-22

### Documentation

- Show demo GIF in README and fix crates.io badge links
Uncomment demo GIF reference and update badge URLs to point to
  rgx-cli on crates.io.

### Features

- Add demo GIF and update dependencies
Generate demo GIF using VHS showing real-time matching, engine
  switching, and flag toggles. Update Cargo.lock after dependency
  bumps from merged dependabot PRs.


## [0.1.1] - 2026-02-22

### Features

- Initial release of rgx — regex101 for the terminal
Interactive TUI with real-time matching, 3 regex engines (Rust regex,
  fancy-regex, PCRE2), capture group highlighting with distinct colors,
  plain-English explanation engine, flag toggles, stdin pipe support,
  and cross-platform support.

  Includes full CI/CD automation (test matrix, clippy, fmt, coverage,
  release-plz, cargo-dist), dependabot config, and issue templates.
