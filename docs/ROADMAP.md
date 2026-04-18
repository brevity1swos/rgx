# rgx Roadmap

## Current Direction (updated 2026-04-18, post-v0.11.0)

**v0.11.0 shipped — grex overlay AND `rgx filter` sub-mode.**

Road A as framed on 2026-04-11 expected v0.11.0 to be the "grex and done" farewell release. In practice, `rgx filter` landed in the same cycle because it was low cost to add and opened a genuinely new use-case surface (piping into agx / sift toolchains). The original Road A spirit — avoid a 2–3 month ripgrep replacement fight — still holds. The filter subcommand is a bounded feature, not a grep replacement: it loads input into memory, wants a single pattern per session, and intentionally lacks the ripgrep flags that would invite feature creep.

**Decision gate before the next feature commit:**

1. **Filter Scope C** — `--json <path>` JSONL-field extraction for deeper agx synergy (~1-2 days)
2. **Filter polish** — match-span highlighting in the results pane, UTF-8-lossy input mode, `--max-lines` safety cap (~1 day total)
3. **Genuine maintenance mode** — stop feature work, reinvest capacity in revenue-generating SaaS projects. Matches original Road A.

Until a choice is made, rgx is in de-facto maintenance mode.

**What "maintenance mode" continues to mean:**
- Active monitoring of GitHub issues and external PRs; respond within a reasonable window
- Dependencies kept current (`cargo update`, CI green, MSRV honored)
- **Editor-mode parity is non-negotiable** — any bug that affects the existing pattern-editor workflow gets fixed, even in maintenance. Users adopted rgx for that workflow; regressions there are the one thing we cannot ship.
- Dogfood occasionally to catch regressions

## v0.11.0 — shipped 2026-04-18

**Two major features + a pre-existing-code clippy fix:**

- **grex overlay (Ctrl+X)** — user enters example strings one per line; rgx calls the `grex` crate and drops the generated pattern into the main editor. Three flag toggles (digit, anchors, case-insensitive). Debounced 150ms regeneration on `spawn_blocking` with a generation counter for stale-result suppression. Broadens the audience beyond regex experts; compounds with the existing recipe library and codegen.
- **`rgx filter` subcommand** — live-filter stdin or a file through a regex with a grep-like TUI: pattern pane + filtered match list + flag toggles. Supports `-v/--invert`, `-c/--count`, `-n/--line-number`, `-i/--case-insensitive`, `-f/--file`. Non-TTY stdout auto-skips the TUI for clean piping (`agx session.jsonl --export md | rgx filter error`). Exit codes match grep (0/1/2).
- **`event.rs` refactor** — collapse the per-arm `if tx.send().is_err() { break }` pattern into a single translation match + one send point, to satisfy clippy 1.95's `collapsible_match` lint. Zero behavior change.

Deferred from v0.11.0 (documented as follow-ups, not blocking):
- Shareable permalinks — still open as a future round-out option
- User-saved pattern library — still open
- Demo GIF regeneration — scheduled for next user-driven release window

## Recently Shipped

- **`rgx filter`** — interactive grep mode with stdin/file input, live regex refinement, `--invert`/`--count`/`--line-number` flags, and non-TTY piping. Synergizes with agx and sift pipelines. Shipped 2026-04-18.
- **v0.10.2** — PCRE2 zero-length match offset fix, runtime PCRE2 version detection, syntax highlight token caching, `OverlayState` extraction, action dispatch moved into `App::handle_action()`.
- **Step-Through Debugger (Ctrl+D)** — PCRE2 callout-based step-through debugger with dual-cursor visualization, backtrack markers, heatmap mode, and debug-from-selected-match. No other terminal regex tool has this.
- **Code Generation (Ctrl+G)** — Generate code in 8 languages (Rust, Python, JS, Go, Java, C#, PHP, Ruby). Closes the biggest feature gap vs regex101.
- **Test Suite Mode (--test)** — Validate regex against should-match/should-not-match assertions in TOML files. CI-friendly exit codes.
- **Alternating Match Colors** — Adjacent matches use distinct background colors for visual clarity.
- **Auto Engine Selection** — Detects lookahead, backreferences, recursion and auto-upgrades to the simplest engine that supports them.

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
