# Step-Through Regex Debugger Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a PCRE2 callout-based step-through regex debugger with dual-cursor visualization, backtrack markers, and heatmap mode.

**Architecture:** The debugger uses PCRE2's `AUTO_CALLOUT` to collect execution steps via raw pcre2-sys FFI (the high-level pcre2 crate blocklists callout APIs). A full-screen TUI overlay displays the trace with step navigation. All unsafe code is isolated in `src/engine/pcre2_debug.rs`.

**Tech Stack:** Rust, pcre2-sys (raw FFI), regex-syntax (AST for offset mapping), ratatui (TUI rendering)

---

### Task 1: Data Model — DebugStep, DebugTrace, PatternToken

**Files:**
- Create: `src/engine/pcre2_debug.rs`
- Modify: `src/engine/mod.rs:1-4` (add module declaration)

- [ ] **Step 1: Create `src/engine/pcre2_debug.rs` with data structs**

```rust
//! PCRE2 step-through debugger using AUTO_CALLOUT.
//!
//! All unsafe FFI code for the debugger is contained in this module.

use super::{EngineError, EngineFlags, EngineResult};

/// A single step in the regex engine's execution trace.
#[derive(Debug, Clone)]
pub struct DebugStep {
    /// Index of this step (0-based).
    pub index: usize,
    /// Byte offset in the pattern string (maps to a token via offset_map).
    pub pattern_offset: usize,
    /// Length of the next item in the pattern at this step.
    pub pattern_item_length: usize,
    /// Byte offset in the subject string.
    pub subject_offset: usize,
    /// True if PCRE2_CALLOUT_BACKTRACK flag was set.
    pub is_backtrack: bool,
    /// Capture state: group index -> Option<(start, end)> byte offsets.
    pub captures: Vec<Option<(usize, usize)>>,
    /// Which match attempt this belongs to (increments when start_match changes).
    pub match_attempt: usize,
}

/// A token in the source pattern with its byte range and description.
#[derive(Debug, Clone)]
pub struct PatternToken {
    /// Start byte offset in the source pattern.
    pub start: usize,
    /// End byte offset in the source pattern.
    pub end: usize,
    /// Human-readable description.
    pub description: String,
}

/// Complete debug trace for a pattern/input pair.
#[derive(Debug, Clone)]
pub struct DebugTrace {
    /// All collected steps.
    pub steps: Vec<DebugStep>,
    /// Whether the step limit was hit.
    pub truncated: bool,
    /// Map from pattern byte offsets to source character ranges + descriptions.
    pub offset_map: Vec<PatternToken>,
    /// Per-token hit counts for heatmap (index into offset_map -> count).
    pub heatmap: Vec<u32>,
    /// Total match attempts observed.
    pub match_attempts: usize,
}
```

- [ ] **Step 2: Add module declaration to `src/engine/mod.rs`**

Add after the existing `pub mod pcre2;` line (line 3):

```rust
#[cfg(feature = "pcre2-engine")]
pub mod pcre2_debug;
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check --all-features`
Expected: Compiles with no errors (structs are defined but unused — that's fine for now)

- [ ] **Step 4: Commit**

```bash
git add src/engine/pcre2_debug.rs src/engine/mod.rs
git commit -m "feat(debug): add DebugStep, DebugTrace, PatternToken data structs"
```

---

### Task 2: Offset Map Builder — regex-syntax AST to PatternToken

**Files:**
- Modify: `src/engine/pcre2_debug.rs`

- [ ] **Step 1: Write failing test for offset map builder**

Add to the bottom of `src/engine/pcre2_debug.rs`:

```rust
/// Build offset map by walking the regex-syntax AST.
/// Each leaf AST node becomes a PatternToken with byte offsets from the source pattern.
pub fn build_offset_map(pattern: &str) -> Vec<PatternToken> {
    Vec::new() // stub
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_map_simple_literal() {
        let tokens = build_offset_map("abc");
        assert_eq!(tokens.len(), 3);
        assert_eq!(tokens[0].start, 0);
        assert_eq!(tokens[0].end, 1);
        assert_eq!(tokens[1].start, 1);
        assert_eq!(tokens[1].end, 2);
        assert_eq!(tokens[2].start, 2);
        assert_eq!(tokens[2].end, 3);
    }

    #[test]
    fn test_offset_map_char_class() {
        let tokens = build_offset_map(r"[a-z]+");
        // [a-z] is one token, + is part of the repetition wrapping it
        assert!(!tokens.is_empty());
        // The character class token should start at 0
        assert_eq!(tokens[0].start, 0);
    }

    #[test]
    fn test_offset_map_groups() {
        let tokens = build_offset_map(r"(\d{3})-(\d{4})");
        assert!(!tokens.is_empty());
        // Should contain tokens for \d, {3}, -, \d, {4} at minimum
        // The hyphen literal should be somewhere in the middle
        let hyphen = tokens.iter().find(|t| t.description.contains('-'));
        assert!(hyphen.is_some());
    }

    #[test]
    fn test_offset_map_empty_pattern() {
        let tokens = build_offset_map("");
        assert!(tokens.is_empty());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test --all-features pcre2_debug::tests::test_offset_map_simple_literal`
Expected: FAIL — `tokens.len()` is 0, expected 3

- [ ] **Step 3: Implement `build_offset_map`**

Replace the stub `build_offset_map` function:

```rust
use regex_syntax::ast::parse::Parser;
use regex_syntax::ast::Ast;

/// Build offset map by walking the regex-syntax AST.
/// Each leaf AST node becomes a PatternToken with byte offsets from the source pattern.
pub fn build_offset_map(pattern: &str) -> Vec<PatternToken> {
    let ast = match Parser::new().parse(pattern) {
        Ok(ast) => ast,
        Err(_) => return Vec::new(),
    };
    let mut tokens = Vec::new();
    collect_tokens(&ast, &mut tokens);
    tokens.sort_by_key(|t| t.start);
    tokens.dedup_by_key(|t| t.start);
    tokens
}

fn collect_tokens(ast: &Ast, tokens: &mut Vec<PatternToken>) {
    match ast {
        Ast::Empty(_) => {}
        Ast::Flags(f) => {
            let span = f.span;
            tokens.push(PatternToken {
                start: span.start.offset,
                end: span.end.offset,
                description: "Flags".to_string(),
            });
        }
        Ast::Literal(lit) => {
            tokens.push(PatternToken {
                start: lit.span.start.offset,
                end: lit.span.end.offset,
                description: format!("Literal '{}'", lit.c),
            });
        }
        Ast::Dot(span) => {
            tokens.push(PatternToken {
                start: span.span.start.offset,
                end: span.span.end.offset,
                description: "Any character".to_string(),
            });
        }
        Ast::Assertion(a) => {
            tokens.push(PatternToken {
                start: a.span.start.offset,
                end: a.span.end.offset,
                description: format!("{:?}", a.kind),
            });
        }
        Ast::ClassUnicode(c) => {
            tokens.push(PatternToken {
                start: c.span.start.offset,
                end: c.span.end.offset,
                description: "Unicode class".to_string(),
            });
        }
        Ast::ClassPerl(c) => {
            let name = match c.kind {
                regex_syntax::ast::ClassPerlKind::Digit => "Digit (\\d)",
                regex_syntax::ast::ClassPerlKind::Space => "Whitespace (\\s)",
                regex_syntax::ast::ClassPerlKind::Word => "Word char (\\w)",
            };
            tokens.push(PatternToken {
                start: c.span.start.offset,
                end: c.span.end.offset,
                description: name.to_string(),
            });
        }
        Ast::ClassBracketed(c) => {
            tokens.push(PatternToken {
                start: c.span.start.offset,
                end: c.span.end.offset,
                description: "Character class".to_string(),
            });
        }
        Ast::Repetition(rep) => {
            // Visit the inner AST (the thing being repeated)
            collect_tokens(&rep.ast, tokens);
        }
        Ast::Group(group) => {
            // Visit the inner AST
            collect_tokens(&group.ast, tokens);
        }
        Ast::Alternation(alt) => {
            for a in &alt.asts {
                collect_tokens(a, tokens);
            }
        }
        Ast::Concat(concat) => {
            for a in &concat.asts {
                collect_tokens(a, tokens);
            }
        }
    }
}

/// Find the token index whose byte range contains `offset`.
/// Returns None if no token matches (snaps to nearest if between tokens).
pub fn find_token_at_offset(offset_map: &[PatternToken], offset: usize) -> Option<usize> {
    // Exact containment
    for (i, token) in offset_map.iter().enumerate() {
        if offset >= token.start && offset < token.end {
            return Some(i);
        }
    }
    // Snap to nearest
    if offset_map.is_empty() {
        return None;
    }
    let mut best = 0;
    let mut best_dist = usize::MAX;
    for (i, token) in offset_map.iter().enumerate() {
        let dist = if offset < token.start {
            token.start - offset
        } else {
            offset - token.end
        };
        if dist < best_dist {
            best_dist = dist;
            best = i;
        }
    }
    Some(best)
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --all-features pcre2_debug::tests`
Expected: All 4 tests pass

- [ ] **Step 5: Commit**

```bash
git add src/engine/pcre2_debug.rs
git commit -m "feat(debug): add offset map builder from regex-syntax AST"
```

---

### Task 3: PCRE2 FFI — Callout-Based Debug Match

**Files:**
- Modify: `src/engine/pcre2_debug.rs`

This is the core unsafe FFI code. We compile and match entirely through pcre2-sys, independent of the high-level `pcre2` crate (its `Regex` struct doesn't expose the raw code pointer).

- [ ] **Step 1: Write failing test for `debug_match`**

Add to the tests module in `src/engine/pcre2_debug.rs`:

```rust
    #[test]
    fn test_debug_match_simple() {
        let flags = EngineFlags::default();
        let trace = debug_match("abc", "xabcy", &flags, 10000, 0).unwrap();
        assert!(!trace.steps.is_empty(), "should have steps");
        assert!(!trace.truncated);
        assert!(trace.match_attempts >= 1);
    }

    #[test]
    fn test_debug_match_backtrack() {
        let flags = EngineFlags::default();
        let trace = debug_match("a+b", "aaac", &flags, 10000, 0).unwrap();
        let has_backtrack = trace.steps.iter().any(|s| s.is_backtrack);
        assert!(has_backtrack, "should detect backtracking");
    }

    #[test]
    fn test_debug_match_step_limit() {
        let flags = EngineFlags::default();
        let trace = debug_match("a+b", "aaac", &flags, 5, 0).unwrap();
        assert!(trace.truncated);
        assert_eq!(trace.steps.len(), 5);
    }

    #[test]
    fn test_debug_match_captures() {
        let flags = EngineFlags::default();
        let trace = debug_match("(a)(b)", "ab", &flags, 10000, 0).unwrap();
        // At least one step should have a non-empty capture
        let has_capture = trace.steps.iter().any(|s| s.captures.iter().any(|c| c.is_some()));
        assert!(has_capture, "should capture groups during matching");
    }

    #[test]
    fn test_debug_match_start_offset() {
        let flags = EngineFlags::default();
        let trace = debug_match(r"\d+", "foo 123 bar 456", &flags, 10000, 8).unwrap();
        // Starting from offset 8 ("bar 456"), the first step should be at or after offset 8
        assert!(
            trace.steps[0].subject_offset >= 8,
            "first step should be at or after start_offset"
        );
    }

    #[test]
    fn test_debug_match_empty_pattern() {
        let flags = EngineFlags::default();
        let trace = debug_match("", "test", &flags, 10000, 0).unwrap();
        assert!(trace.steps.is_empty());
    }

    #[test]
    fn test_debug_match_heatmap() {
        let flags = EngineFlags::default();
        let trace = debug_match("abc", "xabcy", &flags, 10000, 0).unwrap();
        // Heatmap should have an entry for each token
        assert_eq!(trace.heatmap.len(), trace.offset_map.len());
        // At least some tokens should have been tried
        assert!(trace.heatmap.iter().any(|&c| c > 0));
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --all-features pcre2_debug::tests::test_debug_match_simple`
Expected: FAIL — `debug_match` function doesn't exist

- [ ] **Step 3: Implement FFI declarations and `debug_match`**

Add to `src/engine/pcre2_debug.rs`, above the `build_offset_map` function:

```rust
use std::ptr;

use libc::c_void;
use pcre2_sys::*;

// --- FFI types and functions blocklisted by pcre2-sys ---

/// PCRE2 callout block passed to the callout function at each step.
/// Layout matches PCRE2 10.x pcre2_callout_block.
#[repr(C)]
struct Pcre2CalloutBlock {
    version: u32,
    callout_number: u32,
    capture_top: u32,
    capture_last: u32,
    offset_vector: *const usize,
    mark: *const u8,
    subject: *const u8,
    subject_length: usize,
    start_match: usize,
    current_position: usize,
    pattern_position: usize,
    next_item_length: usize,
    callout_string_offset: usize,
    callout_string_length: usize,
    callout_string: *const u8,
    callout_flags: u32,
}

const PCRE2_CALLOUT_BACKTRACK: u32 = 2;

extern "C" {
    fn pcre2_set_callout_8(
        mcontext: *mut pcre2_match_context_8,
        callout: Option<
            unsafe extern "C" fn(*mut Pcre2CalloutBlock, *mut c_void) -> i32,
        >,
        callout_data: *mut c_void,
    ) -> i32;
}

// --- Callout collector ---

struct CollectorState {
    steps: Vec<DebugStep>,
    max_steps: usize,
    last_start_match: usize,
    match_attempt: usize,
}

/// The callout function invoked by PCRE2 at each pattern token.
///
/// # Safety
/// Called by PCRE2 engine during matching. `block` is a valid callout block,
/// `data` is a pointer to our `CollectorState`.
unsafe extern "C" fn callout_fn(
    block: *mut Pcre2CalloutBlock,
    data: *mut c_void,
) -> i32 {
    let state = &mut *(data as *mut CollectorState);
    let block = &*block;

    // Check step limit
    if state.steps.len() >= state.max_steps {
        return 1; // abort this match path
    }

    // Track match attempts
    if block.start_match != state.last_start_match {
        state.match_attempt += 1;
        state.last_start_match = block.start_match;
    }

    // Collect capture state
    let cap_count = block.capture_top as usize;
    let mut captures = Vec::with_capacity(cap_count);
    for i in 0..cap_count {
        let start = *block.offset_vector.add(i * 2);
        let end = *block.offset_vector.add(i * 2 + 1);
        if start == usize::MAX {
            captures.push(None);
        } else {
            captures.push(Some((start, end)));
        }
    }

    let is_backtrack = (block.callout_flags & PCRE2_CALLOUT_BACKTRACK) != 0;

    state.steps.push(DebugStep {
        index: state.steps.len(),
        pattern_offset: block.pattern_position,
        pattern_item_length: block.next_item_length,
        subject_offset: block.current_position,
        is_backtrack,
        captures,
        match_attempt: state.match_attempt,
    });

    0 // continue matching
}

// --- Public API ---

/// Run a debug match using PCRE2 AUTO_CALLOUT, collecting execution steps.
///
/// Returns a `DebugTrace` with all steps, offset map, and heatmap data.
/// The `start_offset` parameter allows debugging from a specific position
/// in the subject (e.g., to debug a specific match).
pub fn debug_match(
    pattern: &str,
    subject: &str,
    flags: &EngineFlags,
    max_steps: usize,
    start_offset: usize,
) -> EngineResult<DebugTrace> {
    if pattern.is_empty() {
        return Ok(DebugTrace {
            steps: Vec::new(),
            truncated: false,
            offset_map: Vec::new(),
            heatmap: Vec::new(),
            match_attempts: 0,
        });
    }

    // Build offset map before the debug run
    let offset_map = build_offset_map(pattern);

    // Compile and match via raw FFI
    let (steps, truncated, match_attempts) = unsafe {
        debug_match_ffi(pattern, subject, flags, max_steps, start_offset)?
    };

    // Build heatmap from steps
    let mut heatmap = vec![0u32; offset_map.len()];
    for step in &steps {
        if let Some(idx) = find_token_at_offset(&offset_map, step.pattern_offset) {
            heatmap[idx] += 1;
        }
    }

    Ok(DebugTrace {
        steps,
        truncated,
        offset_map,
        heatmap,
        match_attempts,
    })
}

/// Raw FFI debug match. Compiles with AUTO_CALLOUT, sets callout, runs match.
///
/// # Safety
/// Uses raw pcre2-sys FFI. All pointers are managed within this function scope.
unsafe fn debug_match_ffi(
    pattern: &str,
    subject: &str,
    flags: &EngineFlags,
    max_steps: usize,
    start_offset: usize,
) -> EngineResult<(Vec<DebugStep>, bool, usize)> {
    // Build compile options
    let mut options: u32 = PCRE2_UTF | PCRE2_AUTO_CALLOUT;
    if flags.case_insensitive {
        options |= PCRE2_CASELESS;
    }
    if flags.multi_line {
        options |= PCRE2_MULTILINE;
    }
    if flags.dot_matches_newline {
        options |= PCRE2_DOTALL;
    }
    if flags.unicode {
        options |= PCRE2_UCP;
    }
    if flags.extended {
        options |= PCRE2_EXTENDED;
    }

    // Compile
    let mut error_code: i32 = 0;
    let mut error_offset: usize = 0;
    let code = pcre2_compile_8(
        pattern.as_ptr(),
        pattern.len(),
        options,
        &mut error_code,
        &mut error_offset,
        ptr::null_mut(),
    );
    if code.is_null() {
        return Err(EngineError::CompileError(format!(
            "PCRE2 compile error {} at offset {}",
            error_code, error_offset
        )));
    }

    // Create match data
    let match_data = pcre2_match_data_create_from_pattern_8(code, ptr::null_mut());
    if match_data.is_null() {
        pcre2_code_free_8(code as *mut _);
        return Err(EngineError::MatchError(
            "Failed to create match data".to_string(),
        ));
    }

    // Create match context with callout
    let match_context = pcre2_match_context_create_8(ptr::null_mut());
    if match_context.is_null() {
        pcre2_match_data_free_8(match_data);
        pcre2_code_free_8(code as *mut _);
        return Err(EngineError::MatchError(
            "Failed to create match context".to_string(),
        ));
    }

    let mut collector = CollectorState {
        steps: Vec::new(),
        max_steps,
        last_start_match: usize::MAX,
        match_attempt: 0,
    };

    pcre2_set_callout_8(
        match_context,
        Some(callout_fn),
        &mut collector as *mut CollectorState as *mut c_void,
    );

    // Run match (NO_JIT to preserve callout_flags)
    let subject_bytes = subject.as_bytes();
    let _rc = pcre2_match_8(
        code,
        subject_bytes.as_ptr(),
        subject_bytes.len(),
        start_offset,
        PCRE2_NO_JIT,
        match_data,
        match_context,
    );

    let truncated = collector.steps.len() >= max_steps;
    let match_attempts = collector.match_attempt + 1;

    // Cleanup
    pcre2_match_context_free_8(match_context);
    pcre2_match_data_free_8(match_data);
    pcre2_code_free_8(code as *mut _);

    Ok((collector.steps, truncated, match_attempts))
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --all-features pcre2_debug::tests`
Expected: All tests pass (including the 4 offset map tests and 7 debug_match tests)

- [ ] **Step 5: Run clippy**

Run: `cargo clippy --all-features -- -D warnings`
Expected: No warnings

- [ ] **Step 6: Commit**

```bash
git add src/engine/pcre2_debug.rs
git commit -m "feat(debug): implement PCRE2 callout-based debug_match via raw FFI"
```

---

### Task 4: Action Enum & Key Mapping

**Files:**
- Modify: `src/input/mod.rs`
- Modify: `src/input/vim.rs`

- [ ] **Step 1: Add debugger actions to `Action` enum**

In `src/input/mod.rs`, add before the `Quit` variant (line 56):

```rust
    ToggleDebugger,
```

- [ ] **Step 2: Map `Ctrl+D` in `key_to_action`**

In `src/input/mod.rs`, add after the `GenerateCode` mapping (line 91), before the `ToggleCaseInsensitive` mapping:

```rust
        KeyCode::Char('d') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            Action::ToggleDebugger
        }
```

- [ ] **Step 3: Add `Ctrl+D` to vim global shortcuts**

In `src/input/vim.rs`, in the `is_global_shortcut` function, add `KeyCode::Char('d')` to the ctrl match arm (line 54). The existing list is:

```rust
        return matches!(
            key.code,
            KeyCode::Char('d')
                | KeyCode::Char('e')
                | KeyCode::Char('z')
                // ... rest of existing chars
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo check --all-features`
Expected: Compiles (warning about unused `ToggleDebugger` is fine for now)

- [ ] **Step 5: Add test for vim global shortcut**

In `src/input/vim.rs`, add to the tests module:

```rust
    #[test]
    fn test_ctrl_d_is_global_shortcut() {
        let mut state = VimState::new();
        let action = vim_key_to_action(key_ctrl(KeyCode::Char('d')), &mut state);
        assert_eq!(action, Action::ToggleDebugger);
    }
```

- [ ] **Step 6: Run tests**

Run: `cargo test --all-features vim::tests`
Expected: All vim tests pass

- [ ] **Step 7: Commit**

```bash
git add src/input/mod.rs src/input/vim.rs
git commit -m "feat(debug): add ToggleDebugger action and Ctrl+D shortcut"
```

---

### Task 5: App State & Debugger Methods

**Files:**
- Modify: `src/app.rs`

- [ ] **Step 1: Add debugger state fields to `App` struct**

In `src/app.rs`, add after the `codegen_language_index` field (line 76):

```rust
    pub show_debugger: bool,
    #[cfg(feature = "pcre2-engine")]
    pub debug_trace: Option<crate::engine::pcre2_debug::DebugTrace>,
    pub debug_step: usize,
    pub debug_show_heatmap: bool,
    pub debug_error: Option<String>,
```

- [ ] **Step 2: Initialize debugger fields in `App::new`**

In the `Self { ... }` block in `App::new`, add after `codegen_language_index: 0,` (line 130):

```rust
            show_debugger: false,
            #[cfg(feature = "pcre2-engine")]
            debug_trace: None,
            debug_step: 0,
            debug_show_heatmap: false,
            debug_error: None,
```

- [ ] **Step 3: Add debugger methods**

Add after the `generate_code` method (after line 638):

```rust
    // --- Debugger ---

    #[cfg(feature = "pcre2-engine")]
    pub fn start_debug(&mut self, max_steps: usize) {
        use crate::engine::pcre2_debug;

        let pattern = self.regex_editor.content().to_string();
        let subject = self.test_editor.content().to_string();
        if pattern.is_empty() || subject.is_empty() {
            self.debug_error = Some("Need both a pattern and test string".to_string());
            return;
        }

        // Auto-switch to PCRE2
        if self.engine_kind != EngineKind::Pcre2 {
            self.switch_engine_to(EngineKind::Pcre2);
            self.recompute();
        }

        // Determine start offset: if a match is selected, debug from its position
        let start_offset = if !self.matches.is_empty() && self.selected_match < self.matches.len() {
            self.matches[self.selected_match].start
        } else {
            0
        };

        match pcre2_debug::debug_match(&pattern, &subject, &self.flags, max_steps, start_offset) {
            Ok(trace) => {
                self.debug_trace = Some(trace);
                self.debug_step = 0;
                self.debug_error = None;
                self.show_debugger = true;
            }
            Err(e) => {
                self.debug_error = Some(e.to_string());
            }
        }
    }

    #[cfg(not(feature = "pcre2-engine"))]
    pub fn start_debug(&mut self, _max_steps: usize) {
        self.debug_error = Some("Debugger requires PCRE2 (build with --features pcre2-engine)".to_string());
    }

    pub fn debug_step_forward(&mut self) {
        #[cfg(feature = "pcre2-engine")]
        if let Some(ref trace) = self.debug_trace {
            if self.debug_step + 1 < trace.steps.len() {
                self.debug_step += 1;
            }
        }
    }

    pub fn debug_step_back(&mut self) {
        self.debug_step = self.debug_step.saturating_sub(1);
    }

    pub fn debug_jump_start(&mut self) {
        self.debug_step = 0;
    }

    pub fn debug_jump_end(&mut self) {
        #[cfg(feature = "pcre2-engine")]
        if let Some(ref trace) = self.debug_trace {
            if !trace.steps.is_empty() {
                self.debug_step = trace.steps.len() - 1;
            }
        }
    }

    pub fn debug_next_match(&mut self) {
        #[cfg(feature = "pcre2-engine")]
        if let Some(ref trace) = self.debug_trace {
            let current_attempt = trace.steps.get(self.debug_step)
                .map(|s| s.match_attempt)
                .unwrap_or(0);
            for (i, step) in trace.steps.iter().enumerate().skip(self.debug_step + 1) {
                if step.match_attempt > current_attempt {
                    self.debug_step = i;
                    return;
                }
            }
        }
    }

    pub fn debug_next_backtrack(&mut self) {
        #[cfg(feature = "pcre2-engine")]
        if let Some(ref trace) = self.debug_trace {
            for (i, step) in trace.steps.iter().enumerate().skip(self.debug_step + 1) {
                if step.is_backtrack {
                    self.debug_step = i;
                    return;
                }
            }
        }
    }

    pub fn debug_toggle_heatmap(&mut self) {
        self.debug_show_heatmap = !self.debug_show_heatmap;
    }
```

- [ ] **Step 4: Verify it compiles**

Run: `cargo check --all-features`
Expected: Compiles with no errors

- [ ] **Step 5: Commit**

```bash
git add src/app.rs
git commit -m "feat(debug): add debugger state and methods to App"
```

---

### Task 6: Config — `debug_max_steps` Setting

**Files:**
- Modify: `src/config/settings.rs`

- [ ] **Step 1: Add `debug_max_steps` to `Settings`**

In `src/config/settings.rs`, add after the `vim_mode` field (line 25):

```rust
    #[serde(default = "default_debug_max_steps")]
    pub debug_max_steps: usize,
```

Add the default function after `default_true` (line 42):

```rust
fn default_debug_max_steps() -> usize {
    10_000
}
```

Update `Default` impl, add after `vim_mode: false,` (line 55):

```rust
            debug_max_steps: default_debug_max_steps(),
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check --all-features`
Expected: Compiles

- [ ] **Step 3: Commit**

```bash
git add src/config/settings.rs
git commit -m "feat(debug): add debug_max_steps config setting (default 10000)"
```

---

### Task 7: Debugger UI — `render_debugger`

**Files:**
- Create: `src/ui/debugger.rs`
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Create `src/ui/debugger.rs`**

```rust
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, Paragraph, Wrap},
    Frame,
};

use super::theme;
use super::syntax_highlight;

#[cfg(feature = "pcre2-engine")]
use crate::engine::pcre2_debug::{DebugTrace, PatternToken};

#[cfg(feature = "pcre2-engine")]
pub fn render_debugger(
    frame: &mut Frame,
    area: Rect,
    trace: &DebugTrace,
    current_step: usize,
    show_heatmap: bool,
    pattern: &str,
    subject: &str,
    bt: BorderType,
) {
    // Full-screen overlay
    let overlay = centered_overlay(frame, area, area.width.saturating_sub(4), area.height.saturating_sub(2));

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(bt)
        .border_style(Style::default().fg(theme::RED))
        .title(Span::styled(
            " Debugger (Ctrl+D) ",
            Style::default().fg(theme::TEXT),
        ))
        .style(Style::default().bg(theme::BASE));

    let inner = block.inner(overlay);
    frame.render_widget(block, overlay);

    if trace.steps.is_empty() {
        let msg = Paragraph::new(Line::from(Span::styled(
            "No steps to display",
            Style::default().fg(theme::SUBTEXT),
        )));
        frame.render_widget(msg, inner);
        return;
    }

    let step = &trace.steps[current_step.min(trace.steps.len() - 1)];

    // Layout: pattern, input, step info, token detail, (heatmap), controls
    let heatmap_height = if show_heatmap { 3 } else { 0 };
    let constraints = vec![
        Constraint::Length(3), // pattern
        Constraint::Length(3), // input
        Constraint::Length(2), // step info + token detail
        Constraint::Length(heatmap_height), // heatmap (optional)
        Constraint::Min(0),    // spacer
        Constraint::Length(2), // controls
    ];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(inner);

    // --- Pattern panel ---
    render_pattern_panel(frame, chunks[0], pattern, step, &trace.offset_map);

    // --- Input panel ---
    render_input_panel(frame, chunks[1], subject, step);

    // --- Step info + token detail ---
    render_step_info(frame, chunks[2], trace, step, current_step, subject);

    // --- Heatmap (optional) ---
    if show_heatmap && heatmap_height > 0 {
        render_heatmap(frame, chunks[3], pattern, &trace.heatmap, &trace.offset_map);
    }

    // --- Controls ---
    render_controls(frame, chunks[5]);
}

#[cfg(feature = "pcre2-engine")]
fn render_pattern_panel(
    frame: &mut Frame,
    area: Rect,
    pattern: &str,
    step: &crate::engine::pcre2_debug::DebugStep,
    offset_map: &[PatternToken],
) {
    let mut spans: Vec<Span> = Vec::new();
    spans.push(Span::styled("Pattern  ", Style::default().fg(theme::RED).add_modifier(Modifier::BOLD)));

    let token_idx = crate::engine::pcre2_debug::find_token_at_offset(offset_map, step.pattern_offset);

    // Build character-by-character spans with highlighting
    for (i, ch) in pattern.char_indices() {
        let byte_end = pattern[i..].chars().next().map(|c| i + c.len_utf8()).unwrap_or(pattern.len());
        let is_current = token_idx.map_or(false, |ti| {
            let token = &offset_map[ti];
            i >= token.start && i < token.end
        });

        let style = if is_current {
            Style::default().fg(theme::BASE).bg(theme::YELLOW)
        } else {
            Style::default().fg(theme::TEXT)
        };
        spans.push(Span::styled(ch.to_string(), style));
    }

    let paragraph = Paragraph::new(Line::from(spans));
    frame.render_widget(paragraph, area);
}

#[cfg(feature = "pcre2-engine")]
fn render_input_panel(
    frame: &mut Frame,
    area: Rect,
    subject: &str,
    step: &crate::engine::pcre2_debug::DebugStep,
) {
    let mut spans: Vec<Span> = Vec::new();
    spans.push(Span::styled("Input    ", Style::default().fg(theme::RED).add_modifier(Modifier::BOLD)));

    for (i, ch) in subject.char_indices() {
        let byte_pos = i;
        let style = if byte_pos == step.subject_offset {
            Style::default().fg(theme::BASE).bg(theme::TEAL)
        } else if byte_pos < step.subject_offset
            && step.captures.first().map_or(false, |c| c.is_some())
        {
            // Within matched region so far
            Style::default().fg(theme::BASE).bg(theme::GREEN)
        } else {
            Style::default().fg(theme::TEXT)
        };
        spans.push(Span::styled(ch.to_string(), style));
    }

    let paragraph = Paragraph::new(Line::from(spans));
    frame.render_widget(paragraph, area);
}

#[cfg(feature = "pcre2-engine")]
fn render_step_info(
    frame: &mut Frame,
    area: Rect,
    trace: &DebugTrace,
    step: &crate::engine::pcre2_debug::DebugStep,
    current_step: usize,
    subject: &str,
) {
    let total = trace.steps.len();
    let backtrack_indicator = if step.is_backtrack {
        Span::styled(
            " BACKTRACK ",
            Style::default().fg(theme::BASE).bg(theme::RED),
        )
    } else {
        Span::styled("", Style::default())
    };

    let truncated_indicator = if trace.truncated {
        Span::styled(" [TRUNCATED]", Style::default().fg(theme::PEACH))
    } else {
        Span::styled("", Style::default())
    };

    let step_line = Line::from(vec![
        Span::styled(
            format!("Step {}/{}", current_step + 1, total),
            Style::default().fg(theme::BLUE).add_modifier(Modifier::BOLD),
        ),
        backtrack_indicator,
        Span::styled(
            format!("    Attempt {}/{}", step.match_attempt + 1, trace.match_attempts),
            Style::default().fg(theme::SUBTEXT),
        ),
        truncated_indicator,
    ]);

    // Token description
    let token_desc = crate::engine::pcre2_debug::find_token_at_offset(
        &trace.offset_map,
        step.pattern_offset,
    )
    .map(|i| trace.offset_map[i].description.clone())
    .unwrap_or_else(|| format!("offset {}", step.pattern_offset));

    // Capture state — show actual captured text from subject
    let mut cap_parts: Vec<String> = Vec::new();
    for (i, cap) in step.captures.iter().enumerate().skip(1) {
        match cap {
            Some((s, e)) => {
                let text = subject.get(*s..*e).unwrap_or("?");
                cap_parts.push(format!("${}=\"{}\"", i, text));
            }
            None => cap_parts.push(format!("${}<empty>", i)),
        }
    }
    let cap_str = if cap_parts.is_empty() {
        String::new()
    } else {
        format!("  Captures: {}", cap_parts.join("  "))
    };

    let detail_line = Line::from(vec![
        Span::styled("Token: ", Style::default().fg(theme::SUBTEXT)),
        Span::styled(token_desc, Style::default().fg(theme::TEXT)),
        Span::styled(cap_str, Style::default().fg(theme::GREEN)),
    ]);

    let paragraph = Paragraph::new(vec![step_line, detail_line]);
    frame.render_widget(paragraph, area);
}

#[cfg(feature = "pcre2-engine")]
fn render_heatmap(
    frame: &mut Frame,
    area: Rect,
    pattern: &str,
    heatmap: &[u32],
    offset_map: &[PatternToken],
) {
    let max_heat = heatmap.iter().copied().max().unwrap_or(1).max(1);

    let mut spans: Vec<Span> = Vec::new();
    spans.push(Span::styled("Heatmap  ", Style::default().fg(theme::RED).add_modifier(Modifier::BOLD)));

    for (i, ch) in pattern.char_indices() {
        let token_idx = crate::engine::pcre2_debug::find_token_at_offset(offset_map, i);
        let heat = token_idx.map(|ti| heatmap.get(ti).copied().unwrap_or(0)).unwrap_or(0);
        let intensity = heat as f64 / max_heat as f64;

        let bg = if intensity < 0.33 {
            theme::BLUE
        } else if intensity < 0.66 {
            theme::PEACH
        } else {
            theme::RED
        };

        let style = if heat == 0 {
            Style::default().fg(theme::SUBTEXT)
        } else {
            Style::default().fg(theme::BASE).bg(bg)
        };
        spans.push(Span::styled(ch.to_string(), style));
    }

    let paragraph = Paragraph::new(Line::from(spans));
    frame.render_widget(paragraph, area);
}

fn render_controls(frame: &mut Frame, area: Rect) {
    let lines = vec![
        Line::from(Span::styled(
            " Left/h Prev  Right/l Next  Home/gg Start  End/G Last ",
            Style::default().fg(theme::SUBTEXT),
        )),
        Line::from(Span::styled(
            " m Next match  f Next fail  H Heatmap  Esc/q Close ",
            Style::default().fg(theme::SUBTEXT),
        )),
    ];
    let paragraph = Paragraph::new(lines);
    frame.render_widget(paragraph, area);
}

fn centered_overlay(frame: &mut Frame, area: Rect, max_width: u16, max_height: u16) -> Rect {
    let w = max_width.min(area.width);
    let h = max_height.min(area.height);
    let x = (area.width.saturating_sub(w)) / 2;
    let y = (area.height.saturating_sub(h)) / 2;
    let rect = Rect::new(x, y, w, h);
    frame.render_widget(Clear, rect);
    rect
}
```

- [ ] **Step 2: Register module and add overlay check in `src/ui/mod.rs`**

Add to module declarations (after line 8):

```rust
#[cfg(feature = "pcre2-engine")]
pub mod debugger;
```

In the `render` function, add after the codegen overlay check (after line 111, before `let error_str`):

```rust
    #[cfg(feature = "pcre2-engine")]
    if app.show_debugger {
        if let Some(ref trace) = app.debug_trace {
            debugger::render_debugger(
                frame,
                size,
                trace,
                app.debug_step,
                app.debug_show_heatmap,
                app.regex_editor.content(),
                app.test_editor.content(),
                bt,
            );
            return;
        }
    }
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check --all-features`
Expected: Compiles

- [ ] **Step 4: Commit**

```bash
git add src/ui/debugger.rs src/ui/mod.rs
git commit -m "feat(debug): add full-screen debugger overlay with dual-cursor and heatmap"
```

---

### Task 8: Event Loop — Wire Up Debugger in main.rs

**Files:**
- Modify: `src/main.rs`

- [ ] **Step 1: Add debugger overlay handler**

In `src/main.rs`, after the codegen overlay handler block (after line 274, where `continue;` closes the codegen handler), add:

```rust
                    if app.show_debugger {
                        use crossterm::event::KeyCode;
                        match key.code {
                            KeyCode::Right | KeyCode::Char('l') => app.debug_step_forward(),
                            KeyCode::Left | KeyCode::Char('h') => app.debug_step_back(),
                            KeyCode::Home => app.debug_jump_start(),
                            KeyCode::End => app.debug_jump_end(),
                            KeyCode::Char('G') => app.debug_jump_end(),
                            KeyCode::Char('g') => app.debug_jump_start(), // single g = start (simplified from gg)
                            KeyCode::Char('m') => app.debug_next_match(),
                            KeyCode::Char('f') => app.debug_next_backtrack(),
                            KeyCode::Char('H') => app.debug_toggle_heatmap(),
                            KeyCode::Esc | KeyCode::Char('q') => {
                                app.show_debugger = false;
                            }
                            _ => {}
                        }
                        continue;
                    }
```

- [ ] **Step 2: Handle `ToggleDebugger` action**

In the main `match action` block, add after the `GenerateCode` handler (after line 412):

```rust
                        Action::ToggleDebugger => {
                            if app.show_debugger {
                                app.show_debugger = false;
                            } else {
                                app.start_debug(settings.debug_max_steps);
                            }
                        }
```

Note: `settings` must be accessible. Check that `settings` is already in scope in main — it is, it's loaded near the top of main and used for `vim_mode`, `rounded_borders`, etc.

- [ ] **Step 3: Verify it compiles**

Run: `cargo check --all-features`
Expected: Compiles

- [ ] **Step 4: Commit**

```bash
git add src/main.rs
git commit -m "feat(debug): wire up debugger overlay and Ctrl+D toggle in event loop"
```

---

### Task 9: Help Page Update

**Files:**
- Modify: `src/ui/mod.rs`

- [ ] **Step 1: Add Ctrl+D to help page**

In `src/ui/mod.rs`, in the `build_help_pages` function, add after the `Ctrl+G` shortcut (line 231):

```rust
        shortcut("Ctrl+D", "Step-through regex debugger"),
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check --all-features`
Expected: Compiles

- [ ] **Step 3: Commit**

```bash
git add src/ui/mod.rs
git commit -m "docs: add Ctrl+D debugger shortcut to help page"
```

---

### Task 10: Integration Tests

**Files:**
- Create: `tests/debugger_tests.rs`

- [ ] **Step 1: Create integration test file**

```rust
#![cfg(feature = "pcre2-engine")]

use rgx::engine::pcre2_debug::{build_offset_map, debug_match, find_token_at_offset};
use rgx::engine::EngineFlags;

#[test]
fn test_debug_match_end_to_end() {
    let flags = EngineFlags::default();
    let trace = debug_match(r"\d+", "abc 123 def", &flags, 10000, 0).unwrap();
    assert!(!trace.steps.is_empty());
    assert!(!trace.offset_map.is_empty());
    assert_eq!(trace.heatmap.len(), trace.offset_map.len());
}

#[test]
fn test_catastrophic_backtracking_detection() {
    let flags = EngineFlags::default();
    let trace = debug_match("(a+)+b", "aaaaaaaac", &flags, 50000, 0).unwrap();
    // Should have many steps due to backtracking
    assert!(trace.steps.len() > 100, "expected many steps, got {}", trace.steps.len());
    // Heatmap should show hot spots
    let max_heat = trace.heatmap.iter().copied().max().unwrap_or(0);
    assert!(max_heat > 10, "expected hot heatmap, max was {}", max_heat);
}

#[test]
fn test_offset_map_accuracy() {
    // Pattern: (abc)
    let tokens = build_offset_map(r"(abc)");
    // Should have tokens for a, b, c inside the group
    assert!(!tokens.is_empty());
    // All tokens should be within the pattern length
    for token in &tokens {
        assert!(token.end <= "(abc)".len());
    }
}

#[test]
fn test_find_token_at_offset() {
    let tokens = build_offset_map("abc");
    // Token for 'a' at offset 0
    assert_eq!(find_token_at_offset(&tokens, 0), Some(0));
    // Token for 'b' at offset 1
    assert_eq!(find_token_at_offset(&tokens, 1), Some(1));
    // Token for 'c' at offset 2
    assert_eq!(find_token_at_offset(&tokens, 2), Some(2));
}

#[test]
fn test_debug_with_flags() {
    let mut flags = EngineFlags::default();
    flags.case_insensitive = true;
    let trace = debug_match("abc", "ABC", &flags, 10000, 0).unwrap();
    assert!(!trace.steps.is_empty(), "case-insensitive should match");
}

#[test]
fn test_debug_unicode_pattern() {
    let flags = EngineFlags { unicode: true, ..Default::default() };
    let trace = debug_match(r"\w+", "cafe\u{0301}", &flags, 10000, 0).unwrap();
    assert!(!trace.steps.is_empty());
}
```

- [ ] **Step 2: Run integration tests**

Run: `cargo test --all-features --test debugger_tests`
Expected: All 6 tests pass

- [ ] **Step 3: Commit**

```bash
git add tests/debugger_tests.rs
git commit -m "test: add integration tests for step-through debugger"
```

---

### Task 11: Full Verification

**Files:** None (verification only)

- [ ] **Step 1: Run full test suite**

Run: `cargo test --all-features`
Expected: All tests pass (existing 210 + new debugger tests)

- [ ] **Step 2: Run clippy**

Run: `cargo clippy --all-features -- -D warnings`
Expected: No warnings

- [ ] **Step 3: Check formatting**

Run: `cargo fmt --check`
Expected: No formatting issues (run `cargo fmt` if needed)

- [ ] **Step 4: Build release**

Run: `cargo build --release --all-features`
Expected: Compiles successfully

- [ ] **Step 5: Manual smoke test**

Run: `cargo run --all-features -- --text "hello 123 world 456" "\\d+"`
Then press `Ctrl+D` to open debugger. Verify:
- Debugger overlay appears with pattern and input highlighted
- `Right`/`Left` steps through execution
- Backtrack indicator appears on backtrack steps
- `H` toggles heatmap
- `Esc` closes debugger

- [ ] **Step 6: Commit all formatting fixes if any**

```bash
cargo fmt
git add -A
git commit -m "chore: format debugger code"
```
