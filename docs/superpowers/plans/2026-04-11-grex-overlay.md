# grex Overlay Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a new `Ctrl+X` overlay that lets users enter example strings and get back a regex pattern via the `grex` crate, loaded into the main editor on Tab. Ships as rgx v0.11.0, the final polish release before maintenance mode (Road A).

**Architecture:** Pure wrapper around `grex::RegExpBuilder` in a new `grex_integration` module; overlay state stored on `OverlayState` as `Option<GrexOverlayState>` (deliberate departure from the existing flat-field pattern because grex state is richer than prior overlays); debounced ~150ms regeneration via `tokio::task::spawn_blocking` with an mpsc result channel and a generation counter for stale-result suppression; overlay-specific key handling in main.rs following the existing `if app.overlay.X { ... }` pattern.

**Tech Stack:** Rust 1.74 MSRV, `grex = "1"` crate, `ratatui` 0.30 for rendering, `tokio` 1 for async runtime and `spawn_blocking`, `insta` for snapshot tests. Existing rgx modules (`input::editor::Editor`, `ui::centered_overlay`, `ui::mod::render_*_overlay` pattern, `syntax_highlight::highlight`) reused as-is.

**Related:**
- Design spec: `docs/superpowers/specs/2026-04-11-grex-overlay-design.md`
- Road A decision: `docs/ROADMAP.md` and memory `decision_v011_road_a.md`

---

## Scope check

Single-feature plan, no decomposition needed. One overlay, one crate dependency, one new module pair (`grex_integration` + `ui::grex_overlay`), plus small modifications to `app.rs`, `input/mod.rs`, `ui/mod.rs`, `main.rs`.

## File structure

**New files:**
- `src/grex_integration.rs` — pure wrapper around the `grex` crate. Exposes `GrexOptions` struct and `generate(examples: &[String], options: GrexOptions) -> String` function. No async, no I/O.
- `src/ui/grex_overlay.rs` — overlay state struct `GrexOverlayState` and `render(frame, area, state)` function. Follows the `src/ui/debugger.rs` precedent of putting larger overlays in their own file.
- `tests/grex_tests.rs` — unit tests for the wrapper, snapshot tests for the overlay, and one end-to-end test.

**Modified files:**
- `Cargo.toml` — add `grex = "1"` to `[dependencies]`.
- `src/lib.rs` — add `pub mod grex_integration;` for test access.
- `src/input/mod.rs` — add `Action::OpenGrex` variant and `Ctrl+X` binding in `key_to_action`.
- `src/app.rs` — add `pub grex: Option<ui::grex_overlay::GrexOverlayState>` field to `OverlayState`; add `grex_result_tx: UnboundedSender<(u64, String)>` and matching receiver to `App`; implement `Action::OpenGrex` arm in `handle_action`; implement grex debounce logic in the existing `App::tick` path.
- `src/main.rs` — add `if let Some(grex_state) = &mut app.overlay.grex { ... }` block before the existing overlay blocks to handle Tab/Esc/Alt+d/Alt+a/Alt+c/plain keys.
- `src/ui/mod.rs` — add `pub mod grex_overlay;`; add a conditional `render_grex_overlay` call alongside the existing `render_recipe_overlay` etc.; add `Ctrl+X` line to `build_help_pages` page 0.
- `src/input/vim.rs` — no code change, but verify via tests that `Ctrl+X` is handled by the existing `is_global_shortcut` / modifier bypass path.
- `README.md` — add grex to the feature list (final polish task).
- `CLAUDE.md` — add `src/grex_integration.rs` and `src/ui/grex_overlay.rs` to the architecture file tree (final polish task).

---

## Task 1: Add grex dependency + create GrexOptions struct

**Files:**
- Modify: `Cargo.toml`
- Create: `src/grex_integration.rs`
- Modify: `src/lib.rs`
- Test: `tests/grex_tests.rs` (new file)

- [ ] **Step 1: Add grex to Cargo.toml**

```toml
# in [dependencies] section
grex = "1"
```

- [ ] **Step 2: Run `cargo fetch` to resolve the dependency**

Run: `cargo fetch`
Expected: PASS, no errors, `grex` listed in Cargo.lock.

- [ ] **Step 3: Create `src/grex_integration.rs` with the `GrexOptions` struct**

```rust
//! Pure wrapper around the `grex` crate for regex generation from example strings.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GrexOptions {
    pub digit: bool,
    pub anchors: bool,
    pub case_insensitive: bool,
}

impl Default for GrexOptions {
    fn default() -> Self {
        Self {
            digit: true,
            anchors: true,
            case_insensitive: false,
        }
    }
}

pub fn generate(examples: &[String], options: GrexOptions) -> String {
    if examples.is_empty() {
        return String::new();
    }
    // Placeholder — real implementation in Task 2.
    let _ = options;
    String::new()
}
```

- [ ] **Step 4: Declare the module in `src/lib.rs`**

Add to `src/lib.rs`:
```rust
pub mod grex_integration;
```

- [ ] **Step 5: Create `tests/grex_tests.rs` with the first failing test**

```rust
use rgx::grex_integration::{generate, GrexOptions};

#[test]
fn default_options_match_spec_defaults() {
    let opts = GrexOptions::default();
    assert!(opts.digit);
    assert!(opts.anchors);
    assert!(!opts.case_insensitive);
}

#[test]
fn empty_input_returns_empty_string() {
    let result = generate(&[], GrexOptions::default());
    assert_eq!(result, "");
}
```

- [ ] **Step 6: Run the tests**

Run: `cargo test --test grex_tests`
Expected: PASS — both tests pass against the stub (`generate` returns `""` for empty input, `GrexOptions::default()` has the documented values).

- [ ] **Step 7: Commit**

```bash
git add Cargo.toml Cargo.lock src/grex_integration.rs src/lib.rs tests/grex_tests.rs
git commit -m "feat(grex): add grex dependency and GrexOptions struct"
```

---

## Task 2: Implement `generate()` with grex RegExpBuilder

**Files:**
- Modify: `src/grex_integration.rs`
- Modify: `tests/grex_tests.rs`

> **Note on grex API:** The plan uses best-guess method names based on grex's README (`RegExpBuilder::from(&slice).with_conversion_of_digits().build()`). If any method name is wrong, run `cargo doc --open -p grex` or `cargo tree -e features -p grex` and update. The semantics are well-documented; only the exact method spelling may drift.

- [ ] **Step 1: Write the failing test for a single-example literal**

Add to `tests/grex_tests.rs`:
```rust
#[test]
fn single_example_with_defaults_is_anchored_literal() {
    let examples = vec!["hello".to_string()];
    let result = generate(&examples, GrexOptions::default());
    assert!(result.starts_with('^'));
    assert!(result.ends_with('$'));
    assert!(result.contains("hello"));
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test --test grex_tests single_example_with_defaults_is_anchored_literal`
Expected: FAIL — current stub returns `""`.

- [ ] **Step 3: Implement `generate()` using `grex::RegExpBuilder`**

Replace the stub body in `src/grex_integration.rs`:
```rust
use grex::RegExpBuilder;

pub fn generate(examples: &[String], options: GrexOptions) -> String {
    if examples.is_empty() {
        return String::new();
    }
    let mut builder = RegExpBuilder::from(examples);
    if options.digit {
        builder.with_conversion_of_digits();
    }
    if !options.anchors {
        builder.without_anchors();
    }
    if options.case_insensitive {
        builder.with_case_insensitive_matching();
    }
    builder.build()
}
```

- [ ] **Step 4: Run the test**

Run: `cargo test --test grex_tests single_example_with_defaults_is_anchored_literal`
Expected: PASS.

If it fails with compile error on a method name, run `cargo doc --open -p grex`, locate `RegExpBuilder` in the generated docs, and update the method spelling. Common alternates: `with_digit_conversion`, `without_anchoring`, `with_case_insensitivity`.

- [ ] **Step 5: Add a digit-flag test**

```rust
#[test]
fn digit_flag_generates_digit_class() {
    let examples = vec!["a1".to_string(), "b22".to_string(), "c333".to_string()];
    let result = generate(&examples, GrexOptions { digit: true, anchors: true, case_insensitive: false });
    assert!(result.contains(r"\d"), "expected \\d in {}", result);
}
```

Run: `cargo test --test grex_tests digit_flag_generates_digit_class`
Expected: PASS (after Step 3 implementation is correct).

- [ ] **Step 6: Add an anchors-off test**

```rust
#[test]
fn anchors_off_produces_unanchored_pattern() {
    let examples = vec!["hello".to_string()];
    let result = generate(&examples, GrexOptions { digit: false, anchors: false, case_insensitive: false });
    assert!(!result.starts_with('^'), "expected no leading ^ in {}", result);
    assert!(!result.ends_with('$'), "expected no trailing $ in {}", result);
}
```

Run: `cargo test --test grex_tests anchors_off_produces_unanchored_pattern`
Expected: PASS.

- [ ] **Step 7: Add a case-insensitive test**

```rust
#[test]
fn case_insensitive_flag_adds_case_modifier() {
    let examples = vec!["Hello".to_string(), "HELLO".to_string(), "hello".to_string()];
    let result = generate(&examples, GrexOptions { digit: false, anchors: true, case_insensitive: true });
    assert!(result.contains("(?i)"), "expected (?i) in {}", result);
}
```

Run: `cargo test --test grex_tests case_insensitive_flag_adds_case_modifier`
Expected: PASS.

- [ ] **Step 8: Run the whole test suite to make sure nothing else broke**

Run: `cargo test --all-features`
Expected: all tests pass.

- [ ] **Step 9: Run clippy**

Run: `cargo clippy --all-features -- -D warnings`
Expected: no warnings.

- [ ] **Step 10: Commit**

```bash
git add src/grex_integration.rs tests/grex_tests.rs
git commit -m "feat(grex): implement generate() wrapper with flag handling"
```

---

## Task 3: Create `GrexOverlayState` struct + `Option` field on `OverlayState`

**Files:**
- Create: `src/ui/grex_overlay.rs`
- Modify: `src/ui/mod.rs`
- Modify: `src/app.rs`
- Test: `tests/grex_tests.rs`

- [ ] **Step 1: Create `src/ui/grex_overlay.rs`**

```rust
//! grex overlay widget — lets users enter example strings and load a generated regex into the main editor.

use std::time::Instant;

use crate::grex_integration::GrexOptions;
use crate::input::editor::Editor;

pub struct GrexOverlayState {
    pub editor: Editor,
    pub options: GrexOptions,
    pub generated_pattern: Option<String>,
    pub generation_counter: u64,
    pub debounce_deadline: Option<Instant>,
}

impl Default for GrexOverlayState {
    fn default() -> Self {
        Self {
            editor: Editor::default(),
            options: GrexOptions::default(),
            generated_pattern: None,
            generation_counter: 0,
            debounce_deadline: None,
        }
    }
}
```

- [ ] **Step 2: Declare the module in `src/ui/mod.rs`**

Add near the top of `src/ui/mod.rs` (before `pub mod debugger;`):
```rust
pub mod grex_overlay;
```

- [ ] **Step 3: Add the field to `OverlayState` in `src/app.rs`**

Locate `pub struct OverlayState { ... }` at `src/app.rs:37` and add the `grex` field. Note: because `Option<GrexOverlayState>` does not satisfy `Default` if `Editor` doesn't either, confirm `Editor::default()` exists (it should — used in other overlays). If `OverlayState` currently uses `#[derive(Default)]`, it stays derived:

```rust
#[derive(Default)]
pub struct OverlayState {
    pub help: bool,
    pub help_page: usize,
    pub recipes: bool,
    pub recipe_index: usize,
    pub benchmark: bool,
    pub codegen: bool,
    pub codegen_language_index: usize,
    pub grex: Option<crate::ui::grex_overlay::GrexOverlayState>,
}
```

If derive breaks because `GrexOverlayState` doesn't implement `Default`, remove `#[derive(Default)]` from `OverlayState`, implement `Default` manually, or make `grex` a plain `Option` (which derives `Default` as `None`). The last option is the cleanest — `Option<T>` always derives `Default` as `None` regardless of whether `T: Default`. No manual impl needed.

- [ ] **Step 4: Write a test confirming `OverlayState::default()` has `grex = None`**

Add to `tests/grex_tests.rs`:
```rust
use rgx::app::OverlayState;

#[test]
fn overlay_state_default_has_no_grex_overlay() {
    let overlay = OverlayState::default();
    assert!(overlay.grex.is_none());
}
```

- [ ] **Step 5: Run the test**

Run: `cargo test --test grex_tests overlay_state_default_has_no_grex_overlay`
Expected: PASS.

Note: if `OverlayState` is not currently `pub` in `src/lib.rs`, export it (`pub use app::OverlayState;`) or access through an existing pub path.

- [ ] **Step 6: Run the full suite**

Run: `cargo test --all-features`
Expected: all tests pass, no existing test breaks because of the new field.

- [ ] **Step 7: Commit**

```bash
git add src/ui/grex_overlay.rs src/ui/mod.rs src/app.rs src/lib.rs tests/grex_tests.rs
git commit -m "feat(grex): add GrexOverlayState with default constructor"
```

---

## Task 4: Add `Action::OpenGrex` variant + `Ctrl+X` binding

**Files:**
- Modify: `src/input/mod.rs`
- Test: `tests/grex_tests.rs`

- [ ] **Step 1: Write the failing test**

Add to `tests/grex_tests.rs`:
```rust
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use rgx::input::{key_to_action, Action};

#[test]
fn ctrl_x_maps_to_open_grex() {
    let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL);
    assert_eq!(key_to_action(key), Action::OpenGrex);
}
```

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test --test grex_tests ctrl_x_maps_to_open_grex`
Expected: FAIL — `Action::OpenGrex` variant does not exist yet.

- [ ] **Step 3: Add the variant to the `Action` enum**

In `src/input/mod.rs`, add `OpenGrex` to the `Action` enum (alphabetically near `OpenRecipes`):
```rust
    OpenGrex,
    OpenRecipes,
```

- [ ] **Step 4: Add the key mapping in `key_to_action`**

Add the binding (alphabetically near the existing Ctrl+R recipes binding at `src/input/mod.rs:87`):
```rust
        KeyCode::Char('x') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::OpenGrex,
```

- [ ] **Step 5: Run the test**

Run: `cargo test --test grex_tests ctrl_x_maps_to_open_grex`
Expected: PASS.

- [ ] **Step 6: Run clippy + full suite**

Run: `cargo clippy --all-features -- -D warnings && cargo test --all-features`
Expected: no warnings, all tests pass.

- [ ] **Step 7: Commit**

```bash
git add src/input/mod.rs tests/grex_tests.rs
git commit -m "feat(grex): bind Ctrl+X to Action::OpenGrex"
```

---

## Task 5: Implement `App::handle_action(OpenGrex)`

**Files:**
- Modify: `src/app.rs`
- Test: `tests/grex_tests.rs`

- [ ] **Step 1: Write the failing test**

Add to `tests/grex_tests.rs`:
```rust
use rgx::app::App;
use rgx::input::Action;

#[test]
fn open_grex_action_opens_overlay() {
    let mut app = App::new_for_tests();
    assert!(app.overlay.grex.is_none());
    app.handle_action(Action::OpenGrex, /* debug_max_steps */ 10_000);
    assert!(app.overlay.grex.is_some());
}
```

Note: if `App::new_for_tests()` does not exist, use whichever constructor existing tests use (check `tests/ui_tests.rs` or `tests/engine_tests.rs` for the pattern). If tests use `App::default()` or a custom helper, match it.

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test --test grex_tests open_grex_action_opens_overlay`
Expected: FAIL — no match arm for `Action::OpenGrex`, the action dispatches to the `None` fallthrough and the overlay stays closed.

- [ ] **Step 3: Add the match arm in `handle_action`**

Locate `pub fn handle_action(&mut self, action: Action, debug_max_steps: usize)` at `src/app.rs:787`. Add a new match arm (near the existing `Action::OpenRecipes` arm at line 871):

```rust
            Action::OpenGrex => {
                self.overlay.grex = Some(crate::ui::grex_overlay::GrexOverlayState::default());
            }
```

- [ ] **Step 4: Run the test**

Run: `cargo test --test grex_tests open_grex_action_opens_overlay`
Expected: PASS.

- [ ] **Step 5: Run full suite + clippy**

Run: `cargo test --all-features && cargo clippy --all-features -- -D warnings`
Expected: all pass, no warnings.

- [ ] **Step 6: Commit**

```bash
git add src/app.rs tests/grex_tests.rs
git commit -m "feat(grex): handle Action::OpenGrex to open the overlay"
```

---

## Task 6: Render empty-state grex overlay

**Files:**
- Modify: `src/ui/grex_overlay.rs`
- Modify: `src/ui/mod.rs` (export + integration)
- Test: `tests/grex_tests.rs`

- [ ] **Step 1: Write the snapshot test for empty state**

Add to `tests/grex_tests.rs`:
```rust
use ratatui::backend::TestBackend;
use ratatui::Terminal;
use rgx::ui::grex_overlay::{render, GrexOverlayState};

#[test]
fn grex_overlay_empty_snapshot() {
    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let state = GrexOverlayState::default();
    terminal
        .draw(|f| {
            let area = f.size();
            render(f, area, &state);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend());
}
```

Note: confirm the existing `tests/ui_tests.rs` pattern for snapshotting a `TestBackend`. Adjust imports and macro invocation to match.

- [ ] **Step 2: Run it to verify it fails (or panics)**

Run: `cargo test --test grex_tests grex_overlay_empty_snapshot`
Expected: FAIL or COMPILE ERROR — `render` function doesn't exist yet.

- [ ] **Step 3: Implement the `render` function (empty state only)**

In `src/ui/grex_overlay.rs`:
```rust
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::ui::centered_overlay;

pub fn render(frame: &mut Frame, area: Rect, state: &GrexOverlayState) {
    let width = area.width.min(80).max(60);
    let height = area.height.saturating_sub(6).min(18).max(12);
    let popup = centered_overlay(frame, area, width, height);

    let dim = Style::default().fg(Color::DarkGray);

    let editor_content = if state.editor.content().is_empty() {
        vec![Line::from(Span::styled(
            "Enter one example per line. Tab to accept.",
            dim,
        ))]
    } else {
        state
            .editor
            .content()
            .lines()
            .map(|l| Line::from(l.to_string()))
            .collect()
    };

    let pattern_line = match &state.generated_pattern {
        Some(p) if !p.is_empty() => Line::from(p.as_str()),
        _ => Line::from(Span::styled("(none yet)", dim)),
    };

    let flag_row = build_flag_row(&state.options);

    let body: Vec<Line> = std::iter::once(Line::from(""))
        .chain(std::iter::once(flag_row))
        .chain(std::iter::once(Line::from("")))
        .chain(std::iter::once(Line::from("Examples (one per line):")))
        .chain(editor_content.into_iter())
        .chain(std::iter::once(Line::from("")))
        .chain(std::iter::once(Line::from("Generated pattern:")))
        .chain(std::iter::once(pattern_line))
        .chain(std::iter::once(Line::from("")))
        .chain(std::iter::once(Line::from(
            "Alt+d/a/c: toggle flags    Tab: accept    Esc: cancel",
        )))
        .collect();

    let block = Block::default()
        .title(" Generate Regex from Examples ")
        .borders(Borders::ALL);
    let paragraph = Paragraph::new(body).block(block);

    frame.render_widget(paragraph, popup);
}

fn build_flag_row(options: &GrexOptions) -> Line<'static> {
    fn flag_span(label: &'static str, on: bool) -> Vec<Span<'static>> {
        let marker = if on { "●" } else { "○" };
        let style = if on {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        vec![
            Span::raw(label),
            Span::raw(" "),
            Span::styled(marker, style),
        ]
    }
    let mut spans = vec![Span::raw("Flags:  ")];
    spans.extend(flag_span("[D]igit", options.digit));
    spans.push(Span::raw("   "));
    spans.extend(flag_span("[A]nchors", options.anchors));
    spans.push(Span::raw("   "));
    spans.extend(flag_span("[C]ase-insensitive", options.case_insensitive));
    Line::from(spans)
}
```

Also add the required import at the top of `src/ui/grex_overlay.rs`:
```rust
use crate::grex_integration::GrexOptions;
```

(Already imported for the struct definition; confirm it covers both uses.)

- [ ] **Step 4: Run the test**

Run: `cargo test --test grex_tests grex_overlay_empty_snapshot`
Expected: FAIL on first run (insta creates a pending snapshot). Review the pending snapshot with `cargo insta review`; accept if the rendering matches the layout in the spec (dimmed placeholder visible, dimmed `(none yet)`, correct flag row).

- [ ] **Step 5: Accept the snapshot and re-run**

Run: `cargo insta accept && cargo test --test grex_tests grex_overlay_empty_snapshot`
Expected: PASS.

- [ ] **Step 6: Full suite + clippy**

Run: `cargo test --all-features && cargo clippy --all-features -- -D warnings`
Expected: all pass.

- [ ] **Step 7: Commit**

```bash
git add src/ui/grex_overlay.rs tests/grex_tests.rs tests/snapshots/
git commit -m "feat(grex): render empty grex overlay with placeholder"
```

---

## Task 7: Wire `render_grex_overlay` into `ui::render` + help page update

**Files:**
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Add the conditional render call**

In `src/ui/mod.rs`, locate the existing overlay render chain around line 92 where `if app.overlay.help { ... }` etc. appear. Add (order: after codegen, before debugger):

```rust
    if let Some(grex_state) = app.overlay.grex.as_ref() {
        grex_overlay::render(frame, size, grex_state);
    }
```

- [ ] **Step 2: Update help page 0 to mention `Ctrl+X`**

Locate `build_help_pages` in `src/ui/mod.rs` and add a new line in the keybindings section of page 0:

```rust
    "Ctrl+X    Generate regex from examples",
```

(Insert near the existing `Ctrl+R    Open recipe library` line.)

- [ ] **Step 3: Run the full suite**

Run: `cargo test --all-features`
Expected: all tests pass, including any existing help-page snapshot tests that may need re-acceptance. If a snapshot test now shows a diff because of the new help line, run `cargo insta review`, verify the diff is just the new Ctrl+X line, and accept.

- [ ] **Step 4: Clippy**

Run: `cargo clippy --all-features -- -D warnings`
Expected: no warnings.

- [ ] **Step 5: Commit**

```bash
git add src/ui/mod.rs tests/snapshots/
git commit -m "feat(grex): wire grex overlay into main render chain and help page"
```

---

## Task 8: Overlay key handler in main.rs

**Files:**
- Modify: `src/main.rs`
- Modify: `src/input/editor.rs` (only if a helper method is needed; likely not)
- Test: `tests/grex_tests.rs`

- [ ] **Step 1: Write an integration test for Tab-with-pattern loading**

Add to `tests/grex_tests.rs`:
```rust
#[test]
fn tab_with_generated_pattern_loads_into_regex_editor_and_closes_overlay() {
    let mut app = App::new_for_tests();
    app.handle_action(Action::OpenGrex, 10_000);
    // Directly seed a generated pattern on the overlay.
    if let Some(overlay) = app.overlay.grex.as_mut() {
        overlay.generated_pattern = Some("^hello$".to_string());
    }
    // Dispatch the Tab key through the same path main.rs uses.
    app.dispatch_grex_overlay_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
    assert!(app.overlay.grex.is_none());
    assert_eq!(app.regex_editor.content(), "^hello$");
}
```

Note: `App::dispatch_grex_overlay_key` is a thin testability helper added in Step 3. Avoids duplicating main.rs's event-loop wiring in tests.

- [ ] **Step 2: Run it to verify it fails**

Run: `cargo test --test grex_tests tab_with_generated_pattern_loads_into_regex_editor_and_closes_overlay`
Expected: FAIL (or COMPILE ERROR) — `dispatch_grex_overlay_key` does not exist yet.

- [ ] **Step 3: Add the dispatch helper on `App`**

In `src/app.rs`, add a new method on `App` (public so tests can reach it):

```rust
pub fn dispatch_grex_overlay_key(&mut self, key: crossterm::event::KeyEvent) {
    use crossterm::event::{KeyCode, KeyModifiers};
    let Some(overlay) = self.overlay.grex.as_mut() else {
        return;
    };
    match key.code {
        KeyCode::Esc => {
            self.overlay.grex = None;
        }
        KeyCode::Tab => {
            if let Some(pattern) = overlay
                .generated_pattern
                .as_ref()
                .filter(|p| !p.is_empty())
                .cloned()
            {
                self.regex_editor.set_content(&pattern);
                self.overlay.grex = None;
                self.recompute();
            }
        }
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::ALT) => {
            overlay.options.digit = !overlay.options.digit;
            overlay.debounce_deadline = Some(std::time::Instant::now() + std::time::Duration::from_millis(150));
        }
        KeyCode::Char('a') if key.modifiers.contains(KeyModifiers::ALT) => {
            overlay.options.anchors = !overlay.options.anchors;
            overlay.debounce_deadline = Some(std::time::Instant::now() + std::time::Duration::from_millis(150));
        }
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::ALT) => {
            overlay.options.case_insensitive = !overlay.options.case_insensitive;
            overlay.debounce_deadline = Some(std::time::Instant::now() + std::time::Duration::from_millis(150));
        }
        _ => {
            // Dispatch ordinary keystrokes to the overlay editor.
            // Use the same editor action mapping used by the main test_editor.
            let action = crate::input::key_to_action(key);
            overlay.editor.apply_action(action);
            overlay.debounce_deadline = Some(std::time::Instant::now() + std::time::Duration::from_millis(150));
        }
    }
}
```

Note: `Editor::apply_action` is the existing method used by other panels. If the existing editor dispatch is named differently (e.g. `Editor::handle_action`), use that name instead. Verify by grepping `src/input/editor.rs`.

- [ ] **Step 4: Verify `Editor::set_content` exists or add it**

Run: `grep -n 'fn set_content\|pub fn content' src/input/editor.rs`
Expected: either `set_content` exists already, or we need to add it. If missing, add:
```rust
pub fn set_content(&mut self, s: &str) {
    // Replace entire content and reset cursor to end. Pushes to undo stack.
    self.push_undo_snapshot();
    self.text = s.to_string();
    self.cursor = self.text.len();
}
```
(Exact field names depend on the current `Editor` struct — verify by reading the top of `src/input/editor.rs`.)

- [ ] **Step 5: Wire the dispatch in `src/main.rs`**

Find the existing overlay key-handling chain in `src/main.rs` (around line 200 where `if app.overlay.help { ... }` starts). Add a new block **before** the help/recipes/benchmark/codegen blocks:

```rust
            if app.overlay.grex.is_some() {
                app.dispatch_grex_overlay_key(key);
                continue;
            }
```

- [ ] **Step 6: Run the test**

Run: `cargo test --test grex_tests tab_with_generated_pattern_loads_into_regex_editor_and_closes_overlay`
Expected: PASS.

- [ ] **Step 7: Add tests for Esc, each Alt toggle, and plain-character editing**

```rust
#[test]
fn esc_closes_grex_overlay_without_loading() {
    let mut app = App::new_for_tests();
    app.handle_action(Action::OpenGrex, 10_000);
    if let Some(overlay) = app.overlay.grex.as_mut() {
        overlay.generated_pattern = Some("should not be loaded".to_string());
    }
    app.dispatch_grex_overlay_key(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE));
    assert!(app.overlay.grex.is_none());
    assert_ne!(app.regex_editor.content(), "should not be loaded");
}

#[test]
fn alt_d_toggles_digit_flag() {
    let mut app = App::new_for_tests();
    app.handle_action(Action::OpenGrex, 10_000);
    let before = app.overlay.grex.as_ref().unwrap().options.digit;
    app.dispatch_grex_overlay_key(KeyEvent::new(KeyCode::Char('d'), KeyModifiers::ALT));
    let after = app.overlay.grex.as_ref().unwrap().options.digit;
    assert_ne!(before, after);
}

#[test]
fn alt_a_toggles_anchors_flag() {
    let mut app = App::new_for_tests();
    app.handle_action(Action::OpenGrex, 10_000);
    let before = app.overlay.grex.as_ref().unwrap().options.anchors;
    app.dispatch_grex_overlay_key(KeyEvent::new(KeyCode::Char('a'), KeyModifiers::ALT));
    let after = app.overlay.grex.as_ref().unwrap().options.anchors;
    assert_ne!(before, after);
}

#[test]
fn alt_c_toggles_case_insensitive_flag() {
    let mut app = App::new_for_tests();
    app.handle_action(Action::OpenGrex, 10_000);
    let before = app.overlay.grex.as_ref().unwrap().options.case_insensitive;
    app.dispatch_grex_overlay_key(KeyEvent::new(KeyCode::Char('c'), KeyModifiers::ALT));
    let after = app.overlay.grex.as_ref().unwrap().options.case_insensitive;
    assert_ne!(before, after);
}

#[test]
fn plain_character_appends_to_overlay_editor() {
    let mut app = App::new_for_tests();
    app.handle_action(Action::OpenGrex, 10_000);
    app.dispatch_grex_overlay_key(KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE));
    app.dispatch_grex_overlay_key(KeyEvent::new(KeyCode::Char('i'), KeyModifiers::NONE));
    let editor_content = app.overlay.grex.as_ref().unwrap().editor.content().to_string();
    assert_eq!(editor_content, "hi");
}
```

Run: `cargo test --test grex_tests`
Expected: all new tests pass.

- [ ] **Step 8: Full suite + clippy**

Run: `cargo test --all-features && cargo clippy --all-features -- -D warnings`
Expected: all pass, no warnings.

- [ ] **Step 9: Commit**

```bash
git add src/main.rs src/app.rs src/input/editor.rs tests/grex_tests.rs
git commit -m "feat(grex): implement grex overlay key handler with Tab/Esc/flag toggles"
```

---

## Task 9: Debounce + spawn_blocking + mpsc result channel

**Files:**
- Modify: `src/app.rs`
- Modify: `src/main.rs` (tick wiring)
- Test: `tests/grex_tests.rs`

- [ ] **Step 1: Add mpsc channel fields to `App`**

In `src/app.rs`, add to the `App` struct:
```rust
pub grex_result_tx: tokio::sync::mpsc::UnboundedSender<(u64, String)>,
pub grex_result_rx: tokio::sync::mpsc::UnboundedReceiver<(u64, String)>,
```

In `App::new` (or wherever the struct is constructed), initialize them:
```rust
let (grex_result_tx, grex_result_rx) = tokio::sync::mpsc::unbounded_channel();
// ... then include these in the struct literal
```

- [ ] **Step 2: Add a `maybe_run_grex_generation` method**

```rust
pub fn maybe_run_grex_generation(&mut self) {
    let Some(overlay) = self.overlay.grex.as_mut() else { return; };
    let Some(deadline) = overlay.debounce_deadline else { return; };
    if std::time::Instant::now() < deadline { return; }

    overlay.debounce_deadline = None;
    overlay.generation_counter += 1;
    let counter = overlay.generation_counter;
    let examples: Vec<String> = overlay
        .editor
        .content()
        .lines()
        .map(|l| l.to_string())
        .filter(|l| !l.is_empty())
        .collect();
    let options = overlay.options;
    let tx = self.grex_result_tx.clone();

    tokio::task::spawn_blocking(move || {
        let pattern = crate::grex_integration::generate(&examples, options);
        let _ = tx.send((counter, pattern));
    });
}

pub fn drain_grex_results(&mut self) {
    while let Ok((counter, pattern)) = self.grex_result_rx.try_recv() {
        if let Some(overlay) = self.overlay.grex.as_mut() {
            if counter == overlay.generation_counter {
                overlay.generated_pattern = Some(pattern);
            }
        }
    }
}
```

- [ ] **Step 3: Wire both into the existing tick path**

Find the existing tick handling in `src/main.rs` (likely inside the event loop where `app.tick()` or a similar call runs). Add, immediately after the existing tick call:

```rust
            app.maybe_run_grex_generation();
            app.drain_grex_results();
```

If there is no central tick path yet, add the calls inside the main event loop right after handling a frame render.

- [ ] **Step 4: Write a test that exercises the full debounce cycle**

```rust
use std::time::Duration;

#[tokio::test(start_paused = true)]
async fn typing_then_waiting_produces_generated_pattern() {
    let mut app = App::new_for_tests();
    app.handle_action(Action::OpenGrex, 10_000);

    // Type three examples
    for line in ["foo", "bar", "baz"] {
        for ch in line.chars() {
            app.dispatch_grex_overlay_key(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
        }
        app.dispatch_grex_overlay_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    }

    // Advance tokio time past the 150ms debounce window
    tokio::time::advance(Duration::from_millis(200)).await;

    // Trigger the generation check and result drain
    app.maybe_run_grex_generation();

    // Let the spawn_blocking task run
    tokio::task::yield_now().await;
    // Give the scheduler a chance to complete the blocking task
    tokio::time::advance(Duration::from_millis(100)).await;
    tokio::task::yield_now().await;

    app.drain_grex_results();

    let overlay = app.overlay.grex.as_ref().unwrap();
    assert!(
        overlay.generated_pattern.is_some(),
        "expected generated_pattern to be populated after debounce"
    );
    let pattern = overlay.generated_pattern.as_ref().unwrap();
    assert!(!pattern.is_empty());
}
```

Note: `#[tokio::test(start_paused = true)]` requires the `test-util` feature on `tokio`. Verify `Cargo.toml` has `tokio = { version = "1", features = ["full"] }` (the spec says yes). `test-util` may need explicit addition to `[dev-dependencies]`:

```toml
[dev-dependencies]
tokio = { version = "1", features = ["test-util", "macros", "rt-multi-thread"] }
```

If the dev-dependency line is missing, add it. If it already covers `test-util`, skip.

- [ ] **Step 5: Run the test**

Run: `cargo test --test grex_tests typing_then_waiting_produces_generated_pattern`
Expected: PASS. If it hangs (because `spawn_blocking` blocks indefinitely under paused time), remove the `start_paused` attribute and use real `tokio::time::sleep` instead — the test will still run in <300ms.

- [ ] **Step 6: Add a stale-result-dropping test**

```rust
#[test]
fn stale_generation_results_are_dropped() {
    let mut app = App::new_for_tests();
    app.handle_action(Action::OpenGrex, 10_000);
    // Manually advance the counter past any in-flight result
    if let Some(overlay) = app.overlay.grex.as_mut() {
        overlay.generation_counter = 10;
    }
    // Send a stale result through the channel
    app.grex_result_tx.send((5, "stale pattern".to_string())).unwrap();
    app.drain_grex_results();
    let overlay = app.overlay.grex.as_ref().unwrap();
    assert_ne!(overlay.generated_pattern.as_deref(), Some("stale pattern"));
}
```

Run: `cargo test --test grex_tests stale_generation_results_are_dropped`
Expected: PASS.

- [ ] **Step 7: Clippy + full suite**

Run: `cargo clippy --all-features -- -D warnings && cargo test --all-features`
Expected: all pass.

- [ ] **Step 8: Commit**

```bash
git add src/app.rs src/main.rs Cargo.toml tests/grex_tests.rs
git commit -m "feat(grex): debounced spawn_blocking generation with stale-result suppression"
```

---

## Task 10: Populated-state rendering with pattern preview and flag indicators

**Files:**
- Modify: `src/ui/grex_overlay.rs` (already mostly done in Task 6)
- Test: `tests/grex_tests.rs`

- [ ] **Step 1: Add snapshot tests for populated states**

```rust
#[test]
fn grex_overlay_populated_snapshot() {
    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut state = GrexOverlayState::default();
    state.editor.set_content("hello@example.com\nfoo.bar@test.org\nadmin+tag@domain.co.uk");
    state.generated_pattern = Some("^[a-z.+@]+\\.[a-z.]+$".to_string());
    terminal
        .draw(|f| {
            let area = f.size();
            render(f, area, &state);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend());
}

#[test]
fn grex_overlay_all_flags_off_snapshot() {
    let backend = TestBackend::new(80, 20);
    let mut terminal = Terminal::new(backend).unwrap();
    let mut state = GrexOverlayState::default();
    state.options = GrexOptions { digit: false, anchors: false, case_insensitive: false };
    state.editor.set_content("a\nb\nc");
    state.generated_pattern = Some("(?:a|b|c)".to_string());
    terminal
        .draw(|f| {
            let area = f.size();
            render(f, area, &state);
        })
        .unwrap();
    insta::assert_snapshot!(terminal.backend());
}
```

- [ ] **Step 2: Run the tests, review, and accept snapshots**

Run: `cargo test --test grex_tests grex_overlay_populated_snapshot grex_overlay_all_flags_off_snapshot`
Expected: FAIL on first run — new snapshots. Run `cargo insta review`, inspect each, accept if they match the spec layout.

- [ ] **Step 3: Re-run tests**

Run: `cargo insta accept && cargo test --test grex_tests`
Expected: all PASS.

- [ ] **Step 4: Clippy**

Run: `cargo clippy --all-features -- -D warnings`
Expected: no warnings.

- [ ] **Step 5: Commit**

```bash
git add tests/grex_tests.rs tests/snapshots/
git commit -m "test(grex): snapshot tests for populated overlay with flag variants"
```

---

## Task 11: End-to-end roundtrip test

**Files:**
- Test: `tests/grex_tests.rs`

- [ ] **Step 1: Write the full roundtrip test**

```rust
#[tokio::test]
async fn grex_roundtrip_full_flow() {
    let mut app = App::new_for_tests();
    app.handle_action(Action::OpenGrex, 10_000);
    assert!(app.overlay.grex.is_some());

    // Type "foo\nbar\nbaz"
    for line in ["foo", "bar", "baz"] {
        for ch in line.chars() {
            app.dispatch_grex_overlay_key(KeyEvent::new(KeyCode::Char(ch), KeyModifiers::NONE));
        }
        app.dispatch_grex_overlay_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    }

    // Wait past the debounce, run generation, drain results
    tokio::time::sleep(Duration::from_millis(200)).await;
    app.maybe_run_grex_generation();
    // Let the blocking task complete
    tokio::task::yield_now().await;
    tokio::time::sleep(Duration::from_millis(100)).await;
    app.drain_grex_results();

    // Pattern should be populated
    assert!(app.overlay.grex.as_ref().unwrap().generated_pattern.is_some());

    // Press Tab to accept
    app.dispatch_grex_overlay_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));

    // Overlay should be closed, regex editor should contain a grex-generated pattern
    assert!(app.overlay.grex.is_none());
    let content = app.regex_editor.content();
    assert!(!content.is_empty(), "regex editor should contain the generated pattern");
    // Pattern should match all three inputs. We verify this by compiling it against each example.
    let re = regex::Regex::new(content).expect("grex output must be a valid regex");
    assert!(re.is_match("foo"));
    assert!(re.is_match("bar"));
    assert!(re.is_match("baz"));
}
```

- [ ] **Step 2: Run it**

Run: `cargo test --test grex_tests grex_roundtrip_full_flow`
Expected: PASS.

- [ ] **Step 3: Full suite + clippy**

Run: `cargo test --all-features && cargo clippy --all-features -- -D warnings`
Expected: all pass.

- [ ] **Step 4: Commit**

```bash
git add tests/grex_tests.rs
git commit -m "test(grex): end-to-end roundtrip from Ctrl+X to regex editor population"
```

---

## Task 12: Vim mode regression tests

**Files:**
- Test: `tests/vim_tests.rs`

- [ ] **Step 1: Add Ctrl+X vim-normal-mode test**

Add to `tests/vim_tests.rs` (following the existing test pattern in that file):

```rust
#[test]
fn ctrl_x_opens_grex_overlay_in_vim_normal_mode() {
    let mut app = App::new_for_tests_with_vim();
    // Confirm starting state: vim normal mode, no overlay
    assert!(app.overlay.grex.is_none());
    let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::CONTROL);
    // The vim dispatch should pass Ctrl+X through as a global shortcut
    app.dispatch_vim_key(key);
    assert!(app.overlay.grex.is_some());
}

#[test]
fn plain_x_still_deletes_in_vim_normal_mode() {
    let mut app = App::new_for_tests_with_vim();
    app.regex_editor.set_content("hello");
    // Position cursor at start, press plain 'x' in normal mode
    app.regex_editor.set_cursor(0);
    let key = KeyEvent::new(KeyCode::Char('x'), KeyModifiers::NONE);
    app.dispatch_vim_key(key);
    assert_eq!(app.regex_editor.content(), "ello");
    // No overlay side-effect
    assert!(app.overlay.grex.is_none());
}
```

Note: `App::new_for_tests_with_vim()` and `dispatch_vim_key` may already exist; if not, use whatever pattern existing vim tests in `tests/vim_tests.rs` use (read the file to confirm).

- [ ] **Step 2: Run the tests**

Run: `cargo test --test vim_tests ctrl_x plain_x`
Expected: PASS.

- [ ] **Step 3: Full suite + clippy**

Run: `cargo test --all-features && cargo clippy --all-features -- -D warnings`
Expected: all pass.

- [ ] **Step 4: Commit**

```bash
git add tests/vim_tests.rs
git commit -m "test(grex): vim mode regression guards for Ctrl+X and plain x"
```

---

## Task 13: Polish — README, CLAUDE.md, demo GIF

**Files:**
- Modify: `README.md`
- Modify: `CLAUDE.md`
- Modify: `assets/demo.tape`

- [ ] **Step 1: Add grex to the README feature list**

In `README.md`, locate the bullet list of features (near the top) and add:

```markdown
- **Generate regex from examples (Ctrl+X)** — enter example strings, get back a regex via the `grex` crate, loaded straight into the main editor.
```

- [ ] **Step 2: Add the new modules to CLAUDE.md architecture tree**

In `CLAUDE.md`, locate the `src/` file tree and add:

```
    grex_integration.rs # Pure wrapper around the grex crate for regex generation from examples
```

under the top-level src listing, and:

```
    grex_overlay.rs     # grex example-to-regex overlay (Ctrl+X)
```

under the `ui/` listing (alphabetical, between `explanation.rs` and `match_display.rs`).

- [ ] **Step 3: Update demo.tape to include a grex overlay demonstration**

Add a new section to `assets/demo.tape` showing Ctrl+X, typing 2–3 examples, and Tab. Keep it short (5–10 seconds of tape time). Follow the existing demo.tape syntax (vhs commands).

- [ ] **Step 4: Regenerate the demo GIF**

Run: `PATH=$HOME/.cargo/bin:$PATH vhs assets/demo.tape`
Expected: new `assets/demo.gif` file.

- [ ] **Step 5: Visual check**

Open `assets/demo.gif` and confirm the grex section renders correctly.

- [ ] **Step 6: Commit**

```bash
git add README.md CLAUDE.md assets/demo.tape assets/demo.gif
git commit -m "docs(grex): add Ctrl+X to README, architecture docs, and demo GIF"
```

---

## Self-review checklist (done before handoff)

**Spec coverage:**
- Q1 (one-example-per-line text area): Task 3 (editor field) + Task 8 (plain char dispatch) ✓
- Q2 (150ms debounce): Task 8 (deadline set) + Task 9 (check in tick) ✓
- Q3 (three flag toggles with Alt+d/a/c): Task 8 (Alt toggle arms) + Task 10 (snapshots) ✓
- Q4 (Tab accepts, Esc cancels): Task 8 (match arms) + Task 11 (end-to-end) ✓
- Q5 (dimmed placeholder, no computing indicator, no size cap): Task 6 (render function) ✓
- Q6 (Ctrl+X shortcut): Task 4 (binding) + Task 12 (vim regression) ✓
- Generation counter for stale results: Task 9 (counter + drop logic + test) ✓
- Pattern preview with syntax highlight: partially deferred — Task 6 renders the pattern as plain text; syntax highlighting via `syntax_highlight::highlight()` is a nice-to-have that the spec mentions but isn't load-bearing. If it's trivial to add during Task 6 (it is — `syntax_highlight::highlight` is a pure function), include it. Otherwise defer to post-ship.
- Help page update: Task 7 ✓
- README + CLAUDE.md: Task 13 ✓

**Placeholder scan:** one deliberate note in Task 2 step 4 ("verify method names against cargo doc") — not a placeholder, a fallback procedure. Task 3 step 3 notes `Option<T>` deriving Default automatically as a simplification. All code blocks contain complete code. No TBD / TODO / implement-later.

**Type consistency:**
- `GrexOptions` fields used consistently: `digit`, `anchors`, `case_insensitive` (never `case_ignoring`, `ignore_case`, `anchor`, etc.)
- `GrexOverlayState` field names consistent: `editor`, `options`, `generated_pattern`, `generation_counter`, `debounce_deadline`
- Method names consistent: `dispatch_grex_overlay_key`, `maybe_run_grex_generation`, `drain_grex_results`
- Action variant: `Action::OpenGrex` used uniformly
- `app.overlay.grex` accessed as `Option<GrexOverlayState>` everywhere

**One known gap:** the plan assumes `App::new_for_tests()` exists. If it doesn't, Task 5 step 1 needs to be preceded by a refactor step to add it, or tests use whatever existing helper the other test files already use. This is a test infrastructure concern that will surface at Task 5 and can be resolved with a 5-minute fix inline.

---

## Execution options

Plan complete and saved to `docs/superpowers/plans/2026-04-11-grex-overlay.md`. Two execution options:

**1. Subagent-Driven (recommended)** — I dispatch a fresh subagent per task, review between tasks, fast iteration. Each task commit gets reviewed before the next launches.

**2. Inline Execution** — Execute tasks in this session using `executing-plans`, batched execution with checkpoints at ~every 3 tasks for review.

Which approach?
