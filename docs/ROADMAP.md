# rgx Roadmap

## Current direction (updated 2026-04-19)

**rgx v0.12.0 shipped; post-release hardening merged to `main`; release-plz has v0.12.1 queued.**

The previous Road A framing — "ship feature-complete, drop into pure
maintenance, reinvest capacity in SaaS" — still holds for standalone rgx.
What changed since that decision is the **stepwise** positioning: rgx is now
one of three terminal-native step-through debuggers (rgx, agx, sift) that
compose into a review stack for the AI-development workflow. That adds a
small, bounded set of integration hooks to the plan, but does not reopen
the "should rgx keep taking main-project capacity?" question. Stepwise
hooks live or die on whether they make rgx materially more useful *to
existing rgx users* — they are not a reason to expand rgx's audience or
rewrite its core.

### What maintenance mode continues to mean

- Active monitoring of GitHub issues and external PRs; respond within a
  reasonable window.
- Dependencies kept current; CI green; MSRV honored.
- **Editor-mode parity is non-negotiable.** Any bug that affects the
  existing pattern-editor workflow gets fixed, even in maintenance. Users
  adopted rgx for that workflow; regressions there are the one thing we
  cannot ship.
- Dogfood occasionally to catch regressions.

### What the stepwise positioning adds

A small backlog of **bounded, opt-in** integration hooks, each with an
opportunity-cost gate before it ships. A stepwise hook earns a slot only
if (a) a sibling tool actually shipped a code path that calls into rgx,
and (b) the hook doesn't rewire rgx's core. Generic phrasing in public
docs — "JSONL pipelines", "policy-file debugging", "agent-transcript
regex inspection" — so the README and this ROADMAP stay readable to
anyone who found rgx through Terminal Trove / awesome-ratatui / AUR and
has no context on the sibling tools.

## Relationship to the stepwise stack

rgx is the **pattern-match layer** of a three-tool stack:

| Tool | Step-through of… | Read/write posture |
|------|------------------|--------------------|
| **rgx** | regex matches, capture groups, engine behavior | read-only |
| **[agx](https://github.com/brevity1swos/agx)** | agent-turn timelines (Claude Code / Codex / Gemini / OpenAI / LangChain / Vercel AI / OTel GenAI) | read-only |
| **[sift](https://github.com/brevity1swos/sift)** | what the agent wrote before it hits disk | write-through gate |

All three compose at the CLI boundary only — no shared library, no
coordinated release train. Each ships on its own cadence. The suite site
lives at **[stepwise](https://github.com/brevity1swos/stepwise)**. If any
one tool stops earning its place on its own merits, it gets cut; the suite
cannot rescue a tool that isn't working standalone.

## Recently Shipped

- **v0.12.0** (2026-04-18 / release-plz) — grex overlay (Ctrl+X) and
  `rgx filter` subcommand landed together. grex pulls in [grex](https://crates.io/crates/grex)
  behind a debounced 150ms `spawn_blocking` task with stale-result
  suppression; filter reads stdin/file and lets users refine a regex
  against the stream interactively, with non-TTY stdout auto-skipping the
  TUI for clean piping.
- **Post-v0.12.0 hardening pass** (on `main`, v0.12.1 queued) — 10-MiB
  per-line input cap (`MAX_LINE_BYTES`) and bounded cap-detection peek
  prevent a hostile unterminated stream from OOMing before `--max-lines`
  engages; JSONL path supports bracketed string keys (`["hyphen-key"]`,
  `["日本語"]`); `match_haystack` helper is now `pub` for third-party
  integrations; `FilterApp::with_json_extracted` returns `Result` instead
  of panicking; UTF-8-safe slicing in the filter UI can no longer panic
  on a malformed match span; json_path parse errors report the real
  character for non-ASCII input.
- **Filter `--json <PATH>`** — dotted/indexed path language
  (`.msg`, `.steps[0].text`, `["user-id"]`) extracts a JSONL field; the
  regex matches against the extracted string but raw JSON lines are still
  what gets emitted. TUI renders two-line rows (raw JSON + `↳ extracted`)
  on wide terminals, single-line narrow fallback. Parse failures / path
  misses / non-string values are skipped silently.
- **`rgx filter`** — interactive grep mode with stdin/file input, live
  regex refinement, `--invert`/`--count`/`--line-number` flags, non-TTY
  piping.
- **Step-through debugger (Ctrl+D, PCRE2)** — dual-cursor visualization,
  backtrack markers, heatmap mode, debug-from-selected-match.
- **Code generation (Ctrl+G)** — 8 languages (Rust, Python, JS, Go, Java,
  C#, PHP, Ruby).
- **Test suite mode (`--test`)** — TOML should-match/should-not-match
  assertions with CI-friendly exit codes.
- **Auto engine selection** — detects lookahead, backreferences, recursion
  and upgrades to the simplest engine that supports them.
- **Alternating match colors**, **grex overlay**, **workspace save/load**,
  **vim mode**, **mouse**, **regex101 URL export**.

## Open — bounded stepwise integration hooks

Each item below has **two gates** before it ships:

1. *Sibling prerequisite*: the sibling tool has shipped the code path that
   invokes rgx. Until that lands on the sibling's `main`, the hook is not
   on rgx's plan.
2. *Opportunity-cost gate*: estimated work must fit in a single evening,
   add no new runtime deps, and not change any existing CLI surface.

| Hook | Triggered by | Work on rgx side | Gates |
|------|--------------|------------------|-------|
| Seed pattern + stdin text from a parent process | agent-transcript step-through tool shipping the jump-in code path | Verify current `-t` / stdin fallback already covers the shape; document the invocation in usage docs if so. No flag change expected. | Sibling prerequisite **not yet met** — tracked as "proposed" in the sibling repo. |
| Export a refined filter pattern to a named file | AI-write-review policy file format stabilizing on regex rules | One keybind in `rgx filter` TUI (e.g. `Ctrl+O export`) that writes `^pattern$\n` to a path chosen via a filename prompt. No parser changes. | Sibling prerequisite **gated on that tool's Phase 2**. |
| `--output-pattern` already emits the current pattern | already shipped | — | — |

Neither hook is promised. Both get reassessed on the sibling tool's next
release. If either turns out to need more than a one-evening implementation,
it gets parked by the Road A opportunity-cost argument.

## Future considerations (unchanged from prior revision)

| Feature | Impact | Effort |
|---------|--------|--------|
| Theme customization | Medium | Low |
| Import from regex101 URL | Low | Low |
| More engines (JavaScript `RegExp`, Python `re`) | Medium | High |
| User-saved pattern library | Medium | Medium |

These remain as low-priority parking spots. Theme customization is the
most-requested by users who tried rgx on light-background terminals; it
would move up the list if two or more users file the same issue.

## Not planned

- **AI / LLM integration inside rgx.** No pattern-writing assistant, no
  "explain this match with a model", no embedded inference. The grex
  overlay is the ceiling here — it's a deterministic tool, not an LLM.
- **Web version.** The terminal-native property is load-bearing across
  the whole stepwise stack; putting rgx in a browser contradicts the
  positioning.
- **Community pattern-sharing platform.** Hosted component; against the
  zero-hosted principle. If a need emerges, users can share patterns via
  `-w <workspace.toml>` files checked into their own repos.
- **Multi-file replace / ripgrep-scope rewrite.** Explicitly abandoned in
  the 2026-04-11 Road A decision and not reopened by the stepwise
  positioning.

## When to rethink this roadmap

Triggers that should cause a revision (same discipline as the sibling
roadmaps):

1. **A stepwise sibling's release notes ship a code path that calls rgx.**
   Move the relevant hook in "Open" above from gated to scheduled, and do
   the one-evening implementation that release cycle.
2. **Editor-mode regression surfaces in a release.** Non-negotiable fix,
   out-of-cycle patch release if needed.
3. **A dep pinned in `Cargo.toml` ships a breaking change that CI catches.**
   Standard upgrade work; document the version floor bump in CHANGELOG.
4. **MSRV needs to bump.** Only on a minor release, with a CHANGELOG
   entry, driven by a concrete need (not speculative cleanup).
5. **A user-visible bug in `rgx filter` surfaces with `--json`.** Filter
   mode is the newest surface and the most likely place for field reports;
   treat these with the same priority as editor-mode regressions.

The roadmap is a prediction, not a contract. Revisit on every minor
release.
