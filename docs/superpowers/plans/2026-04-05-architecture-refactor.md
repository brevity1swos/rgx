# Architecture Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Reduce App struct from 42 fields to ~15 by extracting cohesive sub-structs, move action dispatch out of main.rs event loop, and cache syntax highlighting tokens.

**Architecture:** Extract `OverlayState`, `ScrollState`, `PatternHistory`, `MatchSelection`, and `StatusMessage` sub-structs from App. Move action dispatch to `App::handle_action()`. Cache syntax highlight tokens on App to avoid re-parsing AST every render frame.

**Tech Stack:** Rust, ratatui

---

### Task 1: Extract OverlayState from App

**Files:**
- Modify: `src/app.rs`
- Modify: `src/main.rs`
- Modify: `src/ui/mod.rs`

Group the 7 overlay-related fields into one struct. Every overlay follows the same pattern: a `show_*` bool + optional state (index, page).

- [ ] **Step 1: Define OverlayState in app.rs**

Add above the `App` struct:

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
}
```

- [ ] **Step 2: Replace individual fields in App struct**

Replace these 7 fields:
```rust
    pub show_help: bool,
    pub help_page: usize,
    pub show_recipes: bool,
    pub recipe_index: usize,
    pub show_benchmark: bool,
    pub show_codegen: bool,
    pub codegen_language_index: usize,
```

With:
```rust
    pub overlay: OverlayState,
```

- [ ] **Step 3: Update App::new() initializer**

Replace the 7 field inits with:
```rust
            overlay: OverlayState::default(),
```

- [ ] **Step 4: Update all references in app.rs**

Search-replace all `self.show_help` → `self.overlay.help`, `self.help_page` → `self.overlay.help_page`, `self.show_recipes` → `self.overlay.recipes`, `self.recipe_index` → `self.overlay.recipe_index`, `self.show_benchmark` → `self.overlay.benchmark`, `self.show_codegen` → `self.overlay.codegen`, `self.codegen_language_index` → `self.overlay.codegen_language_index`.

- [ ] **Step 5: Update all references in main.rs**

Same search-replace with `app.` prefix instead of `self.`.

- [ ] **Step 6: Update all references in ui/mod.rs**

Same search-replace with `app.` prefix. This file reads these fields in `render()` and the overlay render functions.

- [ ] **Step 7: Verify**

Run: `cargo test --all-features && cargo clippy --all-targets --all-features -- -D warnings && cargo check --no-default-features`

- [ ] **Step 8: Commit**

```bash
git add src/app.rs src/main.rs src/ui/mod.rs
git commit -m "refactor: extract OverlayState from App (7 fields → 1 struct)"
```

---

### Task 2: Extract ScrollState, PatternHistory, MatchSelection, StatusMessage

**Files:**
- Modify: `src/app.rs`
- Modify: `src/main.rs` (minimal — mostly reads through App methods)
- Modify: `src/ui/mod.rs` (reads scroll/selection fields)
- Modify: `src/ui/match_display.rs` (reads selection fields)
- Modify: `src/ui/explanation.rs` (reads scroll)
- Modify: `src/config/workspace.rs` (reads pattern_history indirectly)

- [ ] **Step 1: Define sub-structs in app.rs**

Add above the `App` struct:

```rust
#[derive(Default)]
pub struct ScrollState {
    pub match_scroll: u16,
    pub replace_scroll: u16,
    pub explain_scroll: u16,
}

pub struct PatternHistory {
    pub entries: VecDeque<String>,
    pub index: Option<usize>,
    pub temp: Option<String>,
}

impl Default for PatternHistory {
    fn default() -> Self {
        Self {
            entries: VecDeque::new(),
            index: None,
            temp: None,
        }
    }
}

#[derive(Default)]
pub struct MatchSelection {
    pub match_index: usize,
    pub capture_index: Option<usize>,
}

pub struct StatusMessage {
    pub text: Option<String>,
    ticks: u32,
}

impl Default for StatusMessage {
    fn default() -> Self {
        Self {
            text: None,
            ticks: 0,
        }
    }
}
```

- [ ] **Step 2: Replace fields in App struct**

Replace these 11 fields:
```rust
    pub match_scroll: u16,
    pub replace_scroll: u16,
    pub explain_scroll: u16,
    pub pattern_history: VecDeque<String>,
    pub history_index: Option<usize>,
    history_temp: Option<String>,
    pub selected_match: usize,
    pub selected_capture: Option<usize>,
    pub clipboard_status: Option<String>,
    clipboard_status_ticks: u32,
```

With:
```rust
    pub scroll: ScrollState,
    pub history: PatternHistory,
    pub selection: MatchSelection,
    pub status: StatusMessage,
```

- [ ] **Step 3: Update App::new() initializer**

Replace the 10 field inits with:
```rust
            scroll: ScrollState::default(),
            history: PatternHistory::default(),
            selection: MatchSelection::default(),
            status: StatusMessage::default(),
```

- [ ] **Step 4: Update all methods in app.rs**

This is the bulk of the work. Key renames:
- `self.match_scroll` → `self.scroll.match_scroll`
- `self.replace_scroll` → `self.scroll.replace_scroll`
- `self.explain_scroll` → `self.scroll.explain_scroll`
- `self.pattern_history` → `self.history.entries`
- `self.history_index` → `self.history.index`
- `self.history_temp` → `self.history.temp`
- `self.selected_match` → `self.selection.match_index`
- `self.selected_capture` → `self.selection.capture_index`
- `self.clipboard_status` → `self.status.text`
- `self.clipboard_status_ticks` → `self.status.ticks`

Also move `set_status_message()` and `tick_clipboard_status()` to be methods on `StatusMessage`:

```rust
impl StatusMessage {
    pub fn set(&mut self, message: String) {
        self.text = Some(message);
        self.ticks = STATUS_DISPLAY_TICKS;
    }

    pub fn tick(&mut self) -> bool {
        if self.text.is_some() {
            if self.ticks > 0 {
                self.ticks -= 1;
            } else {
                self.text = None;
                return true;
            }
        }
        false
    }
}
```

Update callers: `self.set_status_message(msg)` → `self.status.set(msg)`, `self.tick_clipboard_status()` → `self.status.tick()`.

- [ ] **Step 5: Update references in main.rs, ui/mod.rs, ui/match_display.rs, ui/explanation.rs**

Update field access paths. Key changes:
- `app.selected_match` → `app.selection.match_index`
- `app.selected_capture` → `app.selection.capture_index`
- `app.match_scroll` → `app.scroll.match_scroll`
- `app.explain_scroll` → `app.scroll.explain_scroll`
- `app.clipboard_status` → `app.status.text`
- `app.tick_clipboard_status()` → `app.status.tick()`

- [ ] **Step 6: Update workspace.rs if it references history directly**

Check if `Workspace::from_app` or `apply` read `pattern_history` — if so, update path.

- [ ] **Step 7: Verify**

Run: `cargo test --all-features && cargo clippy --all-targets --all-features -- -D warnings && cargo check --no-default-features`

- [ ] **Step 8: Commit**

```bash
git add src/app.rs src/main.rs src/ui/mod.rs src/ui/match_display.rs src/ui/explanation.rs src/config/workspace.rs
git commit -m "refactor: extract ScrollState, PatternHistory, MatchSelection, StatusMessage from App"
```

---

### Task 3: Extract dispatch_action from main.rs

**Files:**
- Modify: `src/app.rs`
- Modify: `src/main.rs`

Move the 215-line action match statement from main.rs into `App::handle_action()`. This keeps all state mutations on App (where they belong) and makes main.rs a thin event loop.

- [ ] **Step 1: Add handle_action method to App**

Add to `impl App` in `src/app.rs`:

```rust
    /// Handle an action from the event loop. Returns true if the app should
    /// continue processing (i.e., the action was handled and no overlay
    /// intercepted it).
    pub fn handle_action(
        &mut self,
        action: Action,
        #[cfg(feature = "pcre2-engine")] debug_max_steps: usize,
    ) {
        match action {
            Action::Quit => self.should_quit = true,
            Action::OutputAndQuit => {
                self.output_on_quit = true;
                self.should_quit = true;
            }
            Action::SwitchPanel => {
                if self.focused_panel == Self::PANEL_REGEX {
                    self.commit_pattern_to_history();
                }
                self.focused_panel = (self.focused_panel + 1) % Self::PANEL_COUNT;
            }
            Action::SwitchPanelBack => {
                if self.focused_panel == Self::PANEL_REGEX {
                    self.commit_pattern_to_history();
                }
                self.focused_panel =
                    (self.focused_panel + Self::PANEL_COUNT - 1) % Self::PANEL_COUNT;
            }
            Action::SwitchEngine => self.switch_engine(),
            Action::Undo => {
                if self.focused_panel == Self::PANEL_REGEX && self.regex_editor.undo() {
                    self.recompute();
                } else if self.focused_panel == Self::PANEL_TEST
                    && self.test_editor.undo()
                {
                    self.rematch();
                } else if self.focused_panel == Self::PANEL_REPLACE
                    && self.replace_editor.undo()
                {
                    self.rereplace();
                }
            }
            Action::Redo => {
                if self.focused_panel == Self::PANEL_REGEX && self.regex_editor.redo() {
                    self.recompute();
                } else if self.focused_panel == Self::PANEL_TEST
                    && self.test_editor.redo()
                {
                    self.rematch();
                } else if self.focused_panel == Self::PANEL_REPLACE
                    && self.replace_editor.redo()
                {
                    self.rereplace();
                }
            }
            Action::HistoryPrev => {
                if self.focused_panel == Self::PANEL_REGEX {
                    self.history_prev();
                }
            }
            Action::HistoryNext => {
                if self.focused_panel == Self::PANEL_REGEX {
                    self.history_next();
                }
            }
            Action::CopyMatch => {
                if self.focused_panel == Self::PANEL_MATCHES {
                    self.copy_selected_match();
                }
            }
            Action::ToggleWhitespace => {
                self.show_whitespace = !self.show_whitespace;
            }
            Action::ToggleCaseInsensitive => {
                self.flags.toggle_case_insensitive();
                self.recompute();
            }
            Action::ToggleMultiLine => {
                self.flags.toggle_multi_line();
                self.recompute();
            }
            Action::ToggleDotAll => {
                self.flags.toggle_dot_matches_newline();
                self.recompute();
            }
            Action::ToggleUnicode => {
                self.flags.toggle_unicode();
                self.recompute();
            }
            Action::ToggleExtended => {
                self.flags.toggle_extended();
                self.recompute();
            }
            Action::ShowHelp => {
                self.overlay.help = true;
            }
            Action::OpenRecipes => {
                self.overlay.recipes = true;
                self.overlay.recipe_index = 0;
            }
            Action::Benchmark => self.run_benchmark(),
            Action::ExportRegex101 => self.copy_regex101_url(),
            Action::GenerateCode => {
                self.overlay.codegen = true;
                self.overlay.codegen_language_index = 0;
            }
            Action::InsertChar(c) => self.edit_focused(|ed| ed.insert_char(c)),
            Action::InsertNewline => {
                if self.focused_panel == Self::PANEL_TEST {
                    self.test_editor.insert_newline();
                    self.rematch();
                }
            }
            Action::DeleteBack => self.edit_focused(Editor::delete_back),
            Action::DeleteForward => self.edit_focused(Editor::delete_forward),
            Action::MoveCursorLeft => self.move_focused(Editor::move_left),
            Action::MoveCursorRight => self.move_focused(Editor::move_right),
            Action::MoveCursorWordLeft => self.move_focused(Editor::move_word_left),
            Action::MoveCursorWordRight => {
                self.move_focused(Editor::move_word_right)
            }
            Action::ScrollUp => match self.focused_panel {
                Self::PANEL_TEST => self.test_editor.move_up(),
                Self::PANEL_MATCHES => self.select_match_prev(),
                Self::PANEL_EXPLAIN => self.scroll_explain_up(),
                _ => {}
            },
            Action::ScrollDown => match self.focused_panel {
                Self::PANEL_TEST => self.test_editor.move_down(),
                Self::PANEL_MATCHES => self.select_match_next(),
                Self::PANEL_EXPLAIN => self.scroll_explain_down(),
                _ => {}
            },
            Action::MoveCursorHome => self.move_focused(Editor::move_home),
            Action::MoveCursorEnd => self.move_focused(Editor::move_end),
            Action::DeleteCharAtCursor => {
                self.edit_focused(Editor::delete_char_at_cursor)
            }
            Action::DeleteLine => self.edit_focused(Editor::delete_line),
            Action::ChangeLine => self.edit_focused(Editor::clear_line),
            Action::OpenLineBelow => {
                if self.focused_panel == Self::PANEL_TEST {
                    self.test_editor.open_line_below();
                    self.rematch();
                } else {
                    self.vim_state.cancel_insert();
                }
            }
            Action::OpenLineAbove => {
                if self.focused_panel == Self::PANEL_TEST {
                    self.test_editor.open_line_above();
                    self.rematch();
                } else {
                    self.vim_state.cancel_insert();
                }
            }
            Action::MoveToFirstNonBlank => {
                self.move_focused(Editor::move_to_first_non_blank)
            }
            Action::MoveToFirstLine => {
                self.move_focused(Editor::move_to_first_line)
            }
            Action::MoveToLastLine => {
                self.move_focused(Editor::move_to_last_line)
            }
            Action::MoveCursorWordForwardEnd => {
                self.move_focused(Editor::move_word_forward_end)
            }
            Action::EnterInsertMode => {}
            Action::EnterInsertModeAppend => self.move_focused(Editor::move_right),
            Action::EnterInsertModeLineStart => {
                self.move_focused(Editor::move_to_first_non_blank)
            }
            Action::EnterInsertModeLineEnd => self.move_focused(Editor::move_end),
            Action::EnterNormalMode => {
                self.move_focused(Editor::move_left_in_line)
            }
            Action::PasteClipboard => {
                if let Ok(mut cb) = arboard::Clipboard::new() {
                    if let Ok(text) = cb.get_text() {
                        self.edit_focused(|ed| ed.insert_str(&text));
                    }
                }
            }
            Action::ToggleDebugger => {
                #[cfg(feature = "pcre2-engine")]
                if self.debug_session.is_some() {
                    self.close_debug();
                } else {
                    self.start_debug(debug_max_steps);
                }
                #[cfg(not(feature = "pcre2-engine"))]
                self.start_debug(0);
            }
            Action::None => {}
        }
    }
```

Note: `SaveWorkspace` is NOT moved here because it does file I/O which is not an App concern — it stays in main.rs as a special case.

- [ ] **Step 2: Replace the match statement in main.rs**

Replace the entire `match action { ... }` block (lines 299-514) with:

```rust
                    match action {
                        Action::SaveWorkspace => {
                            // Workspace save involves file I/O — stays in main
                            let ws = Workspace::from_app(&app);
                            // ... existing save logic unchanged ...
                        }
                        other => {
                            #[cfg(feature = "pcre2-engine")]
                            app.handle_action(other, settings.debug_max_steps);
                            #[cfg(not(feature = "pcre2-engine"))]
                            app.handle_action(other);
                        }
                    }
```

- [ ] **Step 3: Verify**

Run: `cargo test --all-features && cargo clippy --all-targets --all-features -- -D warnings && cargo check --no-default-features`

- [ ] **Step 4: Commit**

```bash
git add src/app.rs src/main.rs
git commit -m "refactor: move action dispatch from main.rs into App::handle_action()"
```

---

### Task 4: Cache syntax highlight tokens

**Files:**
- Modify: `src/app.rs`
- Modify: `src/ui/regex_input.rs`

Currently `syntax_highlight::highlight()` calls `Parser::new().parse(pattern)` on every render frame. The pattern only changes on keystroke. Cache the parsed tokens on App and rebuild only when the pattern changes.

- [ ] **Step 1: Add cached tokens field to App**

```rust
    pub syntax_tokens: Vec<crate::ui::syntax_highlight::SyntaxToken>,
```

Initialize as `syntax_tokens: Vec::new()` in `App::new()`.

- [ ] **Step 2: Rebuild tokens in recompute()**

In `App::recompute()`, after the pattern is compiled, add:

```rust
        self.syntax_tokens = crate::ui::syntax_highlight::highlight(&pattern);
```

For the empty-pattern early return, add `self.syntax_tokens.clear();`.

- [ ] **Step 3: Update RegexInput to use cached tokens**

In `src/ui/regex_input.rs`, change the widget to receive `&[SyntaxToken]` instead of calling `highlight()` internally. The render call in `ui/mod.rs` passes `&app.syntax_tokens`.

- [ ] **Step 4: Verify**

Run: `cargo test --all-features && cargo clippy --all-targets --all-features -- -D warnings`

- [ ] **Step 5: Commit**

```bash
git add src/app.rs src/ui/regex_input.rs src/ui/mod.rs
git commit -m "perf: cache syntax highlight tokens — avoid re-parsing AST every frame"
```
