# rgx Roadmap

## Current direction (updated 2026-04-19)

**rgx v0.12.1 shipped.** The project is in maintenance mode. The regex-
debugging audience has been served: step-through debugger, grex overlay,
code generation, test suite mode, workspace save/load, vim mode, filter
mode with `--json` JSONL-field extraction, and regex101 URL export have
all shipped. There is no meaningful gap left to close in the core
editor-mode workflow.

### What maintenance mode means

- Active monitoring of GitHub issues and external PRs; respond within a
  reasonable window.
- Dependencies kept current; CI green; MSRV honored.
- **Editor-mode parity is non-negotiable.** Any bug that affects the
  existing pattern-editor workflow gets fixed, even in maintenance.
  Users adopted rgx for that workflow; regressions there are the one
  thing we cannot ship.
- Filter mode (`rgx filter`) is the newest surface and the most likely
  place for field-reported bugs. Treated with the same priority as
  editor-mode regressions.
- Dogfood occasionally to catch regressions.

### What maintenance mode does **not** mean

- No active feature development. New feature requests get labeled and
  parked.
- No marketing push, no community-pattern-sharing platform, no hosted
  components.
- No scope expansion into adjacent tool categories (grep replacement,
  multi-file editor, etc.).

## Recently shipped

- **v0.12.1** (2026-04-19) — bug fix: `rgx -p --engine fancy` with
  lookaround patterns no longer errors out. The `recompute()` path used
  to promote `regex-syntax`'s parse error (which doesn't support
  lookaround) into `self.error` even after the engine had compiled the
  pattern successfully; batch mode then short-circuited on that stale
  error. Now the explain failure stays contained and batch mode
  reflects only real compile errors. Also: `EngineFlags::default()` now
  has `unicode: true` to match the runtime settings default; new
  `to_regex_inline_prefix` helper keeps `wrap_pattern` from emitting
  redundant `(?u)` that can trip fancy-regex's backend routing.
- **v0.12.0** (2026-04-19) — grex overlay (Ctrl+X), `rgx filter`
  subcommand with `--invert` / `--count` / `--line-number` /
  `--case-insensitive` / `--file` / `--json <PATH>` / `--max-lines`,
  match-span highlighting in the results pane, UTF-8-lossy input, the
  10-MiB per-line byte cap, `event.rs` clippy-1.95 refactor.
- **Filter `--json <PATH>`** — dotted/indexed/bracketed-string path
  language (`.msg`, `.steps[0].text`, `["user-id"]`, `.["日本語"]`)
  extracts a JSONL field; the regex matches against the extracted
  string but raw JSON lines are still what gets emitted. TUI renders
  two-line rows (raw JSON + `↳ extracted`) on wide terminals, single-
  line narrow fallback. Parse failures / path misses / non-string
  values are skipped silently.
- **Step-through debugger (Ctrl+D, PCRE2)** — dual-cursor visualization,
  backtrack markers, heatmap mode, debug-from-selected-match.
- **Code generation (Ctrl+G)** — 8 languages (Rust, Python, JS, Go,
  Java, C#, PHP, Ruby).
- **Test suite mode (`--test`)** — TOML should-match/should-not-match
  assertions with CI-friendly exit codes.
- **Auto engine selection** — detects lookahead, backreferences,
  recursion and upgrades to the simplest engine that supports them.
- **Alternating match colors**, **workspace save/load**, **vim mode**,
  **mouse**, **regex101 URL export**.

## Future considerations

Low-priority parking spots. None are scheduled; each would move up the
list if two or more users file the same issue.

| Feature | Impact | Effort |
|---------|--------|--------|
| Theme customization | Medium | Low |
| Import from regex101 URL | Low | Low |
| More engines (JavaScript `RegExp`, Python `re`) | Medium | High |
| User-saved pattern library | Medium | Medium |

## Not planned

- **AI / LLM integration inside rgx.** No pattern-writing assistant,
  no "explain this match with a model", no embedded inference. The
  grex overlay is the ceiling here — it's a deterministic tool, not
  an LLM.
- **Web version.** rgx is terminal-native by design.
- **Community pattern-sharing platform.** Hosted component; against
  the zero-hosted principle. If a need emerges, users can share
  patterns via `-w <workspace.toml>` files checked into their own
  repos.
- **Multi-file replace / ripgrep-scope rewrite.** Out of scope. rgx is
  a single-pattern single-input regex tool.

## When to rethink this roadmap

Triggers that should cause a revision:

1. **Editor-mode regression surfaces in a release.** Non-negotiable
   fix, out-of-cycle patch release if needed.
2. **A user-visible bug in `rgx filter` surfaces with `--json` or
   streaming.** Filter mode is the newest surface and treated with
   the same priority as editor-mode regressions.
3. **A dep pinned in `Cargo.toml` ships a breaking change that CI
   catches.** Standard upgrade work; document the version floor bump
   in CHANGELOG.
4. **MSRV needs to bump.** Only on a minor release, with a CHANGELOG
   entry, driven by a concrete need (not speculative cleanup).

The roadmap is a prediction, not a contract. Revisit on every minor
release.
