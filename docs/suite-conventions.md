<!--
Draft shared conventions for the brevity1swos suite (rgx, agx, sift).
Intended to be copied verbatim into each repo's docs/ directory.
Diverging copies are a smell — fix forward, not by branching the doc.

Intended destination: docs/suite-conventions.md in each of rgx, agx, sift.
Lives here in sift/docs/ as the working draft.
-->

# stepwise suite conventions

Shared UX and integration conventions for **rgx**, **agx**, and **sift**
— the three tools that make up the **stepwise** suite. Maintained by
maintainer discipline, not CI. Review every new feature against this
document before merge.

The user-facing landing page for the suite lives at
[`brevity1swos/stepwise`](https://github.com/brevity1swos/stepwise) (not
yet public); brevity1swos is the GitHub organization, stepwise is the
suite's product name.

**Purpose.** Make the three tools feel coordinated without coupling their
codebases. A user who learns one tool should recognize roughly 80% of
the next one's keys, flags, colors, and CLI surface area. The sharing
happens at the convention level, not the code level — no shared crate,
no shared config directory, no shared release train.

---

## 1. Keybindings (TUI)

Every TUI-bearing tool uses the same key for the same verb.

| Key | Verb | rgx | agx | sift |
|---|---|---|---|---|
| `j` / `↓` | next item | ✓ | ✓ | ✓ |
| `k` / `↑` | prev item | ✓ | ✓ | ✓ |
| `g` / `Home` | first item | ✓ | ✓ | ✓ |
| `G` / `End` | last item | ✓ | ✓ | ✓ |
| `d` / `PgDn` | jump forward | — | ✓ | ✓ |
| `u` / `PgUp` | jump back | — | ✓ | ✓ |
| `q` / `Esc` | quit / dismiss overlay | ✓ | ✓ | ✓ |
| `?` / `F1` | help overlay | ✓ | ✓ | ✓ |
| `/` | search / filter | ✓ | ✓ | ✓ |
| `n` / `N` | next / prev search match | ✓ | ✓ | ✓ |
| `y` | yank to clipboard | ✓ | ✓ | ✓ |
| `:N` | jump to item N | — | ✓ | ✓ |
| `m<c>` / `'<c>` | set / goto bookmark | — | ✓ | — |
| `a` | annotate current item | — | ✓ | (retrofit, see §10) |
| `h` | heatmap toggle | ✓ | ✓ | — |
| `s` | stats overlay | — | ✓ | — |
| `Tab` | cycle panel / 2-pane fallback | ✓ | ✓ | — |

### Tool-specific keys

Single-letter keys owned by one tool and not used cross-tool:

- **rgx:** `Ctrl+D` debugger, `Ctrl+G` codegen, `Ctrl+X` grex overlay,
  `Ctrl+W` whitespace, `Ctrl+S` save workspace, `F2` recipes,
  `Alt+↑/↓` pattern history, `Ctrl+←/→` word movement
- **agx:** `f` filter, `A` annotation list, `b` branch tree (planned)
- **sift:** `Enter` accept, `r` revert, `e` edit post-state in `$EDITOR`,
  `t` → agx timeline jump, `p` → rgx policy debug (planned)

### Cross-tool (named integration) keys

Keys that shell out to a sibling tool. Feature-detected; silently degrade
when the sibling is missing (status-bar hint, not a hard failure).

| Key | From | To | Named flow |
|---|---|---|---|
| `t` | sift review | `agx --jump-to <session>:<step>` | Timeline jump |
| `p` | sift review (planned) | `rgx <policy-rule>` | Policy debug |
| `R` | agx (proposed, Phase 5+) | `rgx <regex-arg>` | Regex lens |

---

## 2. CLI grammar

Same flag, same semantics, across tools that support the concept.

| Flag | Meaning | rgx | agx | sift |
|---|---|---|---|---|
| `--summary` | non-interactive summary → stdout | — | ✓ | ✓ |
| `--export md\|html\|json` | transcript / ledger export | — | ✓ | planned |
| `--debug-unknowns` | format-drift report → stderr | — | ✓ | planned |
| `--bench` | timing info → stderr (hidden) | — | ✓ | — |
| `--no-cost` | suppress cost estimates | — | ✓ | n/a |
| `--completions <shell>` | shell completion script | ✓ | ✓ | ✓ |
| `--version` | print version + exit | ✓ | ✓ | ✓ |
| `--help` / `-h` | print help + exit | ✓ | ✓ | ✓ |

### Subcommand grammar

- `<tool> doctor` — health check, reports siblings detected on `PATH` plus
  their versions. Shipped in sift (planned); retrofit to agx and rgx.
- `<tool> corpus <dir>` — batch analysis over a directory tree. Shipped
  in agx; planned in sift; n/a for rgx.
- `<tool> init` — wire up integration with a host tool (hooks, configs).
  Shipped in sift; n/a for rgx and agx.

### Exit codes

- `0` — success, nothing unusual
- `1` — expected no-result path (no matches, no pending, validation failure)
- `2` — error (bad args, missing file, parse failure, missing required sibling)

---

## 3. Color palette

The conversation palette is shared across agx and sift. rgx's regex-
semantic palette is scoped to regex tokens and doesn't conflict.

| Token | Color | rgx | agx | sift |
|---|---|---|---|---|
| `[user]` / user input | cyan | — | ✓ | ✓ |
| `[asst]` / assistant text | green | — | ✓ | ✓ |
| `[tool]` / tool call | yellow | — | ✓ | ✓ |
| `[result]` / tool result | magenta | — | ✓ | ✓ |
| error / `is_error` | red + bold | ✓ | ✓ | ✓ |
| annotation marker | magenta `*` prefix | — | ✓ | planned |
| batch / fork marker | gray `║` prefix | — | ✓ | — |
| accepted ledger entry | green | — | — | ✓ |
| reverted ledger entry | red | — | — | ✓ |
| pending ledger entry | default | — | — | ✓ |

### rgx regex palette (scoped, no cross-tool meaning)

- matches: cycling background colors (alternating for adjacent matches)
- capture groups: per-group foreground colors
- anchors / quantifiers / character classes: distinct syntax-highlight colors

---

## 4. Text / ASCII conventions

- **No emoji in output.** Ever. Not in logs, not in exports, not in TUI,
  not in help text. Terminal-native principle.
- **ASCII role prefixes:** `[user]`, `[asst]`, `[tool]`, `[result]`.
  Never `👤` `🤖` `🔧` `📤`.
- **Unicode markers are acceptable** when they carry visual meaning with
  no reasonable ASCII equivalent: `↵` newline visualization,
  `·` space visualization, `→` tab visualization, `║` branch marker,
  `*` annotation marker.
- **Em-dashes** (`—`) over hyphens-pairs (`--`) in prose.

---

## 5. File / IPC contracts (public surfaces between tools)

Anything a sibling depends on is a **public surface.** Breaks require a
release note; silent breaks are bugs.

### agx → public for sift

- **`agx --export json <session>`** — stable JSON schema:
  `{totals, steps, annotations?}`. Schema versioning is agx's
  responsibility. sift treats this as its primary session-parser
  dependency.
- **`agx --jump-to <session>:<step>`** — CLI surface. Accepts either a
  session file path or a session ID; step is a 0-indexed integer.
  Must launch the TUI on the specified step.
- **`agx --version`** — stable machine-parseable format
  (`agx X.Y.Z`). Used by sift's `doctor` subcommand, which reads the
  first `X.Y.Z` token. Compiled-in features are reported by
  `agx doctor --json`, not by `--version`.

### rgx → public for sift and agx

- **`rgx -P` / `rgx --output-pattern`** — prints the final pattern to
  stdout on exit; enables `eval $(rgx -P)`.
- **`rgx --print --text <str> <pat>`** — non-interactive match
  check (the pattern is positional; `--test` is rgx's TOML test-suite
  mode, not input text); stdout receives ANSI-colored matches or empty.
  Exit: 0 match, 1 no match, 2 error.
- **`rgx --version`** — stable machine-parseable format (`rgx X.Y.Z`).

### sift → public for agx (shipped, Phase 1.7)

The Phase 1.7 reframe (sift = snapshot oracle, agx = navigator) elevates
sift from "leaf consumer" to a publishing producer that agx (and any
other consumer — eval harnesses, third-party tools) can address. The
contract is sift-side only; agx ships its overlay rendering on its own
cadence. One-way coupling per §6 rule 5 is preserved: this is a second
producer→consumer edge, not a bidirectional dependency.

- **`sift export --session <id> --format json`** — emit the full ledger
  as a stable JSON schema:
  `{sift_export_version: 1, session_id, started_at, ended_at,
  transcript_path, turns: [{turn, entries: [LedgerEntry]}]}`. Schema
  versioned via the integer `sift_export_version`; consumers refuse
  unknown major versions. Schema documented in
  `docs/export-schema.md` (sift repo). Stability commitment dates
  from Phase 1.7 ship.
- **`sift state --session <id> --at-turn <N> --format json`** — return
  the file world at turn N as a `path → SHA-1-hex` map. Agx and other
  consumers compose this twice (turns A and B) to diff the file world
  between any two arbitrary turns — a primitive nothing else in the
  AI-dev workflow exposes (git is commit-grain; the agent transcript
  records intent, not state).
- **`sift --version`** — stable machine-parseable format
  (`sift X.Y.Z`). Used by agx (and rgx) `doctor` subcommands once
  those land.

This surface shipped in Phase 1.7 (2026-04-19): the binary exposes
`sift export --format json` and `sift state --at-turn N --format json`
as documented above, with the schema in `docs/export-schema.md`. The
§10 retrofit table has been updated (sift-side row closed; agx-side
overlay rendering remains open as a downstream consumer).

---

## 6. Integration rules

Every cross-tool integration must follow these rules. New integrations
that don't fit go through a conventions update before shipping.

1. **Feature-detect at runtime.** `which <sibling>` at startup or on
   first key press that needs it. Never require a sibling at install or
   build time.
2. **Silent degrade.** Missing sibling → status-bar hint pointing at the
   install docs. Never crash, never refuse to launch, never error-exit.
3. **Subprocess boundary.** No linked-library dependencies between the
   three tools. Integration is `spawn + wait` on a documented CLI, or a
   read of a documented file format. No shared Rust crates published
   for integration purposes.
4. **Named integrations.** Every cross-tool flow has a name that appears
   identically in both sides' docs: *Timeline jump*, *Policy debug*,
   *Regex lens*, etc. Names appear in help overlays and READMEs.
5. **One-way coupling.** The consumer knows the producer. The producer
   never knows its consumers. rgx knows about nothing. agx knows about
   nothing in the suite. sift knows about agx and rgx.
6. **Version floor, not ceiling.** Integrations declare "requires agx
   ≥ X.Y" and work with any newer version. Break detection is at
   runtime via `<sibling> --version`.
7. **No reciprocal keybinds.** If sift binds `t` → agx, agx does **not**
   bind a reverse key back to sift. Reciprocity would violate rule 5.

---

## 7. Versioning & compatibility

Each tool ships on its own cadence (semver inside each repo). The suite's
cross-tool compatibility is tracked in a **compatibility table** in each
tool's README, updated on release:

```markdown
## Compatibility

| sift | works with agx | works with rgx |
|------|----------------|----------------|
| 0.3.x | 0.4.x+ | 0.11.x+ |
| 0.4.x | 0.5.x+ | 0.11.x+ |
```

When a cross-tool contract breaks:
- The breaking tool's release notes call it out explicitly.
- The consumer tool's next release bumps its version-floor in the table.
- The consumer tool's `doctor` subcommand reports incompatibility and
  suggests the upgrade.

---

## 8. Shared prose & vocabulary

Every README uses these exact phrases where applicable. The repetition is
the point — reads like one team, because it is.

- "Terminal-native."
- "Zero telemetry."
- "No hosted components."
- "Narrow scope, deep engineering."
- "Dual-licensed under MIT OR Apache-2.0."

Each README's structure follows the same shape:

1. Name + one-line pitch
2. Demo GIF
3. "What it is" paragraph
4. Install block
5. Quick Reference / Keys
6. Architecture summary
7. "Pairs well with" — cross-link to the other two tools
8. License

Not every section must appear, but section order is fixed where sections
do appear.

---

## 9. README cross-links

Every README carries a "Pairs well with" section near the bottom:

```markdown
## Pairs well with

- **[rgx](https://github.com/brevity1swos/rgx)** — terminal regex debugger.
  Used by sift for policy-rule debugging.
- **[agx](https://github.com/brevity1swos/agx)** — terminal agent session
  viewer. Sift's `t` keybind jumps into agx for timeline context.

All three tools are independent — each earns its keep alone. Combined,
they form **stepwise**, the terminal-native step-through debugger stack
for the AI-development workflow.
```

Omit self-reference on each tool's own page. Customize the one-liners to
the reading tool's perspective.

---

## 10. Open retrofit items

Known convention drift, tracked for future alignment. Not urgent; closed
opportunistically when the affected code changes for other reasons.

| Item | Target | Current | Notes |
|---|---|---|---|
| rgx `doctor` | report siblings | not shipped | Retrofit from sift's `doctor` design (shipped v0.3). |
| agx overlay rendering | decorate timeline + diff turn pair | not shipped | Downstream consumer of sift's Phase 1.7 substrate. Agx-side; tracked in agx's roadmap. Sift-side requires no coordination — ships on agx's cadence. |

**Closed (shipped):**

- sift publish surface (`sift export --format json` + `sift state
  --at-turn N`): shipped Phase 1.7. §5 sift section moved from
  "planned" to "shipped". agx-side overlay rendering remains open as
  a downstream consumer (row above), needs no sift coordination.
- sift accept key (`Enter` / `Space`): additive alias shipped v0.3;
  legacy `a`=accept removed in v0.4. Recorded in §1.
- sift annotate key (`a`): flipped from `n` in v0.4. Recorded in §1.
- sift search (`/` + `n` / `N`): added in v0.4. Recorded in §1.
- agx `doctor`: shipped in agx v0.3 (`agx doctor [--json]`), reporting
  agx version and features plus agx-mcp, sift and rgx presence.

---

## 11. When to update this document

- **Before** adding a new cross-tool integration — it must appear in §1
  (keys), §5 (contracts), and §6 (rules) first.
- **After** breaking a public surface in §5 — add a version note and
  update the compatibility table in each affected README.
- **When** retrofitting a tool to match a convention it currently
  violates — move the item from §10 into the appropriate earlier section
  and record the retrofit date in the tool's CHANGELOG.
- **Never** by diverging the per-repo copies. If a repo needs to deviate,
  amend this doc first and propagate.

---

## 12. Scope boundary

This document covers **shared conventions** across the three tools. It
does **not** cover:

- Per-tool architecture (lives in each repo's CLAUDE.md)
- Per-tool roadmap (lives in each repo's ROADMAP.md)
- Per-tool release process (lives in each repo's CONTRIBUTING.md)
- Cross-tool build / CI (no such system exists; each repo's CI is its own)

When in doubt: if it's about how two tools talk to each other, it's here.
If it's about how one tool is built internally, it's in that tool's
CLAUDE.md.
