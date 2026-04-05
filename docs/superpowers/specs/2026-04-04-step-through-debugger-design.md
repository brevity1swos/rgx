# Step-Through Regex Debugger — Design Spec

## Overview

A step-through regex debugger for rgx that visualizes how the PCRE2 engine processes a pattern character by character, showing backtracking, match attempts, and a heatmap of token hit frequency. No terminal regex tool offers this — it would be a major differentiator.

Activated via `Ctrl+D`. Renders as a full-screen multi-widget overlay. PCRE2-only (auto-switches engine on activation).

## Decisions

| Decision | Choice | Rationale |
|----------|--------|-----------|
| Engine | PCRE2 `AUTO_CALLOUT` via direct pcre2-sys FFI | Only engine with execution tracing. Same approach as regex101. |
| UI | Full-screen multi-widget overlay | Debugger needs structured layout (dual-cursor, heatmap). Simple `Paragraph` overlay is insufficient. |
| Activation | `Ctrl+D`, context-aware | When a match is selected, debug from that match's start offset. Otherwise debug from position 0. |
| Visualization | Dual-cursor + backtrack markers + toggleable heatmap | Heatmap is the killer diagnostic for catastrophic backtracking. Cheap to compute once steps are collected. |
| Step limit | Configurable, default 10,000 | `debug_max_steps` in `config.toml`. Power users can raise for backtracking diagnosis. |
| Scope | PCRE2-only | Rust `regex` crate and fancy-regex have no introspection. Non-PCRE2 builds show a clear message. |
| Non-goals | Auto-play, breakpoints, mouse interaction, trace export, non-PCRE2 fallback | Future enhancements, not v1. |

## Data Model

### DebugStep

A single step in the regex engine's execution trace, collected from a PCRE2 callout.

```rust
pub struct DebugStep {
    pub index: usize,
    /// Byte offset in the pattern string (maps to a token via offset_map)
    pub pattern_offset: usize,
    /// Byte offset in the subject string
    pub subject_offset: usize,
    /// True if PCRE2_CALLOUT_BACKTRACK flag was set
    pub is_backtrack: bool,
    /// Capture state at this step: group index -> Option<(start, end)> byte offsets
    pub captures: Vec<Option<(usize, usize)>>,
    /// Which match attempt this belongs to (increments when start_match changes)
    pub match_attempt: usize,
}
```

### DebugTrace

Complete debug trace for a pattern/input pair.

```rust
pub struct DebugTrace {
    pub steps: Vec<DebugStep>,
    /// Whether the step limit was hit
    pub truncated: bool,
    /// Map from pattern byte offsets to source character ranges + descriptions
    pub offset_map: Vec<PatternToken>,
    /// Per-token hit counts for heatmap (index into offset_map -> count)
    pub heatmap: Vec<u32>,
    pub match_attempts: usize,
}
```

### PatternToken

A token in the source pattern with its position and description.

```rust
pub struct PatternToken {
    /// Byte range in the source pattern
    pub source_range: (usize, usize),
    /// Human-readable description (from explain/formatter)
    pub description: String,
}
```

### Offset Mapping

PCRE2 callouts report `pattern_position` as a byte offset into the pattern string. Since rgx passes the pattern unchanged to PCRE2, these byte offsets correspond directly to the source pattern.

The offset map is built by walking the `regex-syntax` AST before the debug run. Each leaf AST node (literal, character class, assertion, escape) becomes a `PatternToken` with `source_range` from `ast.span().start.offset..ast.span().end.offset`. This mirrors the existing `syntax_highlight.rs` approach.

At runtime, when a callout fires with `pattern_position = N`, binary search the token list for the token whose `source_range` contains byte offset N. If not found (e.g., PCRE2 synthetic positions), snap to the nearest token.

Edge cases:
- Empty pattern: no tokens, debugger shows "nothing to debug"
- PCRE2-only features (verbs like `(*SKIP)`): not in regex-syntax AST, shown as "unknown token at offset N"
- Unicode patterns: byte offsets are correct since both regex-syntax and PCRE2 operate on UTF-8

## PCRE2 FFI Layer

All unsafe code lives in a single new file: `src/engine/pcre2_debug.rs`, gated behind `#[cfg(feature = "pcre2-engine")]`.

### FFI Declarations

The `pcre2` Rust crate v0.2 intentionally blocklists callout functions from its bindings. We declare them manually against `pcre2-sys`:

```rust
#[repr(C)]
pub struct pcre2_callout_block_8 {
    pub version: u32,
    pub callout_number: u32,
    pub capture_top: u32,
    pub capture_last: u32,
    pub offset_vector: *const usize,
    pub mark: *const u8,
    pub subject: *const u8,
    pub subject_length: usize,
    pub start_match: usize,
    pub current_position: usize,
    pub pattern_position: usize,
    pub next_item_length: usize,
    pub callout_string_offset: usize,
    pub callout_string_length: usize,
    pub callout_string: *const u8,
    pub callout_flags: u32,
}

extern "C" {
    fn pcre2_set_callout_8(
        mcontext: *mut pcre2_match_context_8,
        callout: Option<
            extern "C" fn(*mut pcre2_callout_block_8, *mut c_void) -> i32,
        >,
        callout_data: *mut c_void,
    ) -> i32;
}
```

### Collection Flow

1. Create a `MatchContext` with `pcre2_match_context_create_8(null)`
2. Set callout via `pcre2_set_callout_8` with a pointer to a `CollectorState` struct
3. The `extern "C"` callout function reads the callout block fields, pushes a `DebugStep`, increments heatmap counters, returns 0 (continue) or 1 (abort if step limit hit)
4. Compile pattern with `PCRE2_AUTO_CALLOUT` ORed into options, **without JIT** (JIT zeroes `callout_flags`)
5. Run `pcre2_match_8` with the match context
6. Free the match context
7. Return the collected `DebugTrace`

### Public API

```rust
pub fn debug_match(
    pattern: &str,
    subject: &str,
    flags: &EngineFlags,
    max_steps: usize,
    start_offset: usize,
) -> EngineResult<DebugTrace>
```

All `unsafe` is contained in this one file. The rest of the codebase sees only this safe function and the data structs.

## UI: Full-Screen Debugger Overlay

A new file `src/ui/debugger.rs` with a `render_debugger()` function. Uses `Layout` with constraints for structured multi-widget rendering, not a flat `Paragraph`.

### Layout

```
┌──── Debugger (Ctrl+D) ──────────────────────────┐  border: theme::RED
│                                                   │
│  Pattern                                          │  section label
│  (\d{3})-(\d{4})                                  │  syntax highlighted
│   ^^^^                                            │  current token: YELLOW bg
│                                                   │
│  Input                                            │  section label
│  Call 555-1234 or 800-9999                        │  current position: TEAL bg
│       ^                                           │  matched region: GREEN bg
│                                                   │
│  Step 42/1830          * BACKTRACK    Attempt 1/2 │  RED for backtrack
│  Token: \d{3} — "Three digits"                    │  current token description
│  Captures: $1="555"  $2=<empty>                   │  GREEN values, SUBTEXT empty
│                                                   │
│  Heatmap (hidden by default, toggle H)            │  gradient BLUE->PEACH->RED
│  (\d{3})-(\d{4})                                  │
│  ▓▓████░░░░████░                                  │
│                                                   │
│  Left/h Prev  Right/l Next  Home Start  End Last  │  controls in SUBTEXT
│  m Next match  f Next fail  H Heatmap  Esc Close  │
└───────────────────────────────────────────────────┘
```

### Visual Harmonization

- Accent color: `theme::RED` for border and section labels (follows one-color-per-overlay convention: help=BLUE, recipes=GREEN, benchmark=PEACH, codegen=MAUVE)
- All other colors from existing `theme::*` palette — no new colors
- Pattern syntax highlighting reuses existing `syntax_highlight.rs`
- Capture colors reuse `theme::capture_color()`
- Heatmap gradient: `BLUE` (cold) -> `PEACH` (warm) -> `RED` (hot)
- Border uses `border_type(app.rounded_borders)` for consistency
- Footer controls in `theme::SUBTEXT`, same style as other overlays

### Scrolling

Pattern and input displays scroll horizontally to keep the current position centered when content exceeds terminal width.

### Key Bindings (Debugger Mode)

| Key | Action |
|-----|--------|
| `Right` / `l` | Step forward |
| `Left` / `h` | Step backward |
| `Home` / `gg` | Jump to first step |
| `End` / `G` | Jump to last step |
| `m` | Jump to next successful match |
| `f` | Jump to next failure/backtrack |
| `H` | Toggle heatmap panel |
| `Esc` / `q` | Close debugger |

## App State & Integration

### New Fields in App

```rust
pub show_debugger: bool,
pub debug_trace: Option<DebugTrace>,
pub debug_step: usize,
pub debug_show_heatmap: bool,
pub debug_error: Option<String>,
```

### Activation Flow (Ctrl+D)

1. Check if PCRE2 feature is available. If not, set `debug_error` with message.
2. If `selected_match.is_some()`, use that match's start offset. Otherwise 0.
3. Auto-switch engine to PCRE2 if not already.
4. Call `debug_match(pattern, subject, flags, max_steps, start_offset)`.
5. Store result in `debug_trace`, set `debug_step = 0`, `show_debugger = true`.

### Step Navigation Methods

```rust
pub fn debug_step_forward(&mut self)   // min(step + 1, len - 1)
pub fn debug_step_back(&mut self)      // step.saturating_sub(1)
pub fn debug_jump_start(&mut self)     // step = 0
pub fn debug_jump_end(&mut self)       // step = len - 1
pub fn debug_next_match(&mut self)     // next step where match_attempt changes
pub fn debug_next_backtrack(&mut self) // next step where is_backtrack == true
pub fn debug_toggle_heatmap(&mut self) // flip debug_show_heatmap
```

### New Action Variants

```rust
DebugStepForward,
DebugStepBack,
DebugJumpStart,
DebugJumpEnd,
DebugNextMatch,
DebugNextBacktrack,
DebugToggleHeatmap,
ToggleDebugger,  // Ctrl+D — mapped in global key_to_action()
```

Debugger-specific actions are mapped inside the debugger overlay handler in `main.rs`, not in global `key_to_action()`. Only `ToggleDebugger` (Ctrl+D) is global.

### Event Loop Pattern

```rust
if app.show_debugger {
    match action {
        Action::DebugStepForward => app.debug_step_forward(),
        Action::DebugStepBack => app.debug_step_back(),
        // ... other debug actions
        Action::Quit | Action::CloseOverlay => app.show_debugger = false,
        _ => {}
    }
    continue;
}
```

### Config

```toml
debug_max_steps = 10000
```

Added to `Settings` struct in `src/config/settings.rs`, passed to `debug_match()`.

### Vim Mode

`Ctrl+D` added to `is_global_shortcut()` in `src/input/vim.rs` so it bypasses vim processing in both Normal and Insert modes.

## File Changes

### New Files

| File | Purpose |
|------|---------|
| `src/engine/pcre2_debug.rs` | FFI declarations, `debug_match()`, data structs, offset map builder |
| `src/ui/debugger.rs` | `render_debugger()` with multi-widget layout |
| `tests/debugger_tests.rs` | Integration tests for debug trace correctness |

### Modified Files

| File | Changes |
|------|---------|
| `src/engine/mod.rs` | Re-export `pcre2_debug` module (gated) |
| `src/app.rs` | Debugger state fields, step navigation methods, `start_debug()` |
| `src/input/mod.rs` | `ToggleDebugger` action, `Ctrl+D` mapping |
| `src/ui/mod.rs` | `pub mod debugger`, `show_debugger` check in `render()` |
| `src/main.rs` | Debugger overlay key dispatch, `ToggleDebugger` handler |
| `src/config/settings.rs` | `debug_max_steps` field |
| `src/input/vim.rs` | `Ctrl+D` in `is_global_shortcut()` |

## Testing

### Unit Tests (in `src/engine/pcre2_debug.rs`)

All gated behind `#[cfg(feature = "pcre2-engine")]`:

- `test_simple_match_trace` — pattern `abc`, input `xabcy`. Verify steps exist, offsets are valid.
- `test_backtrack_detection` — pattern `a+b`, input `aaac`. Verify `is_backtrack == true` appears.
- `test_capture_state_progression` — pattern `(a)(b)`, input `ab`. Verify captures grow across steps.
- `test_step_limit_truncation` — `max_steps = 5`. Verify `truncated == true`, `steps.len() == 5`.
- `test_start_offset` — pattern `\d+`, input `foo 123 bar 456`. Start at second match position. Verify trace starts from that offset.
- `test_empty_pattern` — returns empty trace, no crash.
- `test_unicode_offsets` — multibyte input, verify byte offset correctness.
- `test_heatmap_counts` — verify non-zero counts for tried tokens.

### Integration Tests (`tests/debugger_tests.rs`)

- `test_debug_match_api` — end-to-end pattern + input -> `DebugTrace` structure.
- `test_catastrophic_backtracking_detection` — pattern `(a+)+b`, input `aaaaaaaac`. High step count, hot heatmap on `a+`.
- `test_offset_map_accuracy` — verify `PatternToken.source_range` values match actual pattern positions.

### No Snapshot Tests

Step traces depend on PCRE2's internal execution order which may vary across versions. Tests assert structural properties (backtrack exists, captures grow, counts match) rather than exact step sequences.
