# rgx Roadmap

## Current Direction (decided 2026-04-11)

**Road A — ship final polish, declare rgx feature-complete, move on.**

rgx has saturated the "in-memory regex editor" niche at v0.10.2. Step-through debugger, codegen, recipes, test suite mode, workspace save/load, vim mode, regex101 export — everything in the "debug a regex" workflow has shipped. There is no meaningful remaining gap to close in that mode.

The strategic question on 2026-04-11 was whether to pivot rgx into building a daily-driver grep replacement (Road B — 2–3 months of main-project focus building `rgx --grep`) or to round out the niche and exit cleanly (Road A). **Road A won on opportunity cost.** With ~1 day/week of sustained capacity, that time is worth more invested in revenue-generating side projects than in a 2–3 month fight with ripgrep for a niche that likely tops out at 500–1000 stars. rgx at 184 stars on a plateau is a respectable resting place, not a failure — the regex-debugging audience has been served.

**What this means:**
- rgx remains actively maintained: bug fixes, dependency updates, external PR review, and community responses all continue
- New feature development stops after the v0.11.0 polish release
- Active-development capacity reinvests in the revenue-generating side projects queued behind rgx

## v0.11.0 — final polish release

Ship grex integration as the farewell feature. Optionally round out with shareable permalinks or a user-saved pattern library if there is natural energy — but do not bundle them upfront.

**In scope:**
- **grex integration** — new overlay (candidate shortcut: `F3` or `Ctrl+X`) for the "I have strings, give me the pattern" workflow. User enters example strings one per line; rgx calls the `grex` crate and drops the generated pattern into the main editor. Days of work, broadens the audience beyond regex experts, and compounds with the existing recipe library and codegen.

**Optional — only if grex lands well and there is leftover energy:**
- Shareable permalinks — host state in a URL that round-trips back into rgx (closes the last real gap vs regex101.com)
- User-saved pattern library — user patterns alongside the built-in recipes

**Deliberately out of scope (and staying out of scope):**
- Multi-file search/replace with preview
- Any further major features after v0.11.0 (beyond the `rgx filter` sub-mode noted below)

## Post-v0.11.0 — maintenance mode

- Monitor GitHub issues and external PRs; respond within a reasonable window
- Keep dependencies current (`cargo update`, CI green, MSRV honored)
- **Editor-mode parity is non-negotiable** — any bug that affects the existing pattern-editor workflow gets fixed, even in maintenance mode. Users adopted rgx for that workflow; regressions there are the one thing we cannot ship.
- Dogfood occasionally to catch regressions
- No active feature development; new feature requests get labeled and parked

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
