//! PCRE2 step-through debugger using AUTO_CALLOUT.
//!
//! All unsafe FFI code for the debugger is contained in this module.

use std::ffi::c_void;
use std::ptr;

use pcre2_sys::*;

use super::{EngineError, EngineFlags, EngineResult};

#[derive(Debug, Clone)]
pub struct DebugStep {
    pub pattern_offset: usize,
    pub pattern_item_length: usize,
    pub subject_offset: usize,
    pub is_backtrack: bool,
    pub captures: Vec<Option<(usize, usize)>>,
    pub match_attempt: usize,
}

#[derive(Debug, Clone)]
pub struct PatternToken {
    pub start: usize,
    pub end: usize,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct DebugTrace {
    pub steps: Vec<DebugStep>,
    pub truncated: bool,
    pub offset_map: Vec<PatternToken>,
    pub heatmap: Vec<u32>,
    pub match_attempts: usize,
    /// Maps each byte offset in the pattern to its token index
    /// (or `usize::MAX` if none).
    pub byte_to_token: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct DebugSession {
    pub trace: DebugTrace,
    pub step: usize,
    pub show_heatmap: bool,
    pub pattern: String,
    pub subject: String,
}

// pcre2-sys blocklists callout types/functions from its generated bindings,
// so we declare the struct and extern fn manually to match the PCRE2 C ABI.
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

unsafe extern "C" {
    fn pcre2_set_callout_8(
        mcontext: *mut pcre2_match_context_8,
        callout: Option<unsafe extern "C" fn(*mut Pcre2CalloutBlock, *mut c_void) -> i32>,
        callout_data: *mut c_void,
    ) -> i32;
}

const CALLOUT_CONTINUE: i32 = 0;
const CALLOUT_ABORT: i32 = 1;

struct CollectorState {
    steps: Vec<DebugStep>,
    max_steps: usize,
    last_start_match: usize,
    match_attempt: usize,
}

unsafe extern "C" fn callout_fn(block: *mut Pcre2CalloutBlock, data: *mut c_void) -> i32 {
    let state = unsafe { &mut *(data as *mut CollectorState) };
    let block = unsafe { &*block };

    if state.steps.len() >= state.max_steps {
        return CALLOUT_ABORT;
    }

    if block.start_match != state.last_start_match {
        state.match_attempt += 1;
        state.last_start_match = block.start_match;
    }

    let cap_count = block.capture_top as usize;
    let mut captures = Vec::with_capacity(cap_count);
    for i in 0..cap_count {
        let start = unsafe { *block.offset_vector.add(i * 2) };
        let end = unsafe { *block.offset_vector.add(i * 2 + 1) };
        if start == usize::MAX {
            captures.push(None);
        } else {
            captures.push(Some((start, end)));
        }
    }

    let is_backtrack = (block.callout_flags & PCRE2_CALLOUT_BACKTRACK) != 0;

    state.steps.push(DebugStep {
        pattern_offset: block.pattern_position,
        pattern_item_length: block.next_item_length,
        subject_offset: block.current_position,
        is_backtrack,
        captures,
        match_attempt: state.match_attempt,
    });

    CALLOUT_CONTINUE
}

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
            byte_to_token: Vec::new(),
        });
    }

    let offset_map = build_offset_map(pattern);

    let mut byte_to_token = vec![usize::MAX; pattern.len()];
    for (ti, token) in offset_map.iter().enumerate() {
        for slot in byte_to_token
            .iter_mut()
            .take(token.end.min(pattern.len()))
            .skip(token.start)
        {
            *slot = ti;
        }
    }

    let (steps, truncated, match_attempts) =
        unsafe { debug_match_ffi(pattern, subject, flags, max_steps, start_offset)? };

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
        byte_to_token,
    })
}

unsafe fn debug_match_ffi(
    pattern: &str,
    subject: &str,
    flags: &EngineFlags,
    max_steps: usize,
    start_offset: usize,
) -> EngineResult<(Vec<DebugStep>, bool, usize)> {
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

    let mut error_code: i32 = 0;
    let mut error_offset: usize = 0;
    let code = unsafe {
        pcre2_compile_8(
            pattern.as_ptr(),
            pattern.len(),
            options,
            &mut error_code,
            &mut error_offset,
            ptr::null_mut(),
        )
    };
    if code.is_null() {
        return Err(EngineError::CompileError(format!(
            "PCRE2 compile error {} at offset {}",
            error_code, error_offset
        )));
    }

    let match_data = unsafe { pcre2_match_data_create_from_pattern_8(code, ptr::null_mut()) };
    if match_data.is_null() {
        unsafe { pcre2_code_free_8(code) };
        return Err(EngineError::MatchError(
            "Failed to create match data".to_string(),
        ));
    }

    let match_context = unsafe { pcre2_match_context_create_8(ptr::null_mut()) };
    if match_context.is_null() {
        unsafe { pcre2_match_data_free_8(match_data) };
        unsafe { pcre2_code_free_8(code) };
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

    unsafe {
        pcre2_set_callout_8(
            match_context,
            Some(callout_fn),
            &mut collector as *mut CollectorState as *mut c_void,
        )
    };

    let subject_bytes = subject.as_bytes();
    let _rc = unsafe {
        pcre2_match_8(
            code,
            subject_bytes.as_ptr(),
            subject_bytes.len(),
            start_offset,
            PCRE2_NO_JIT,
            match_data,
            match_context,
        )
    };

    let truncated = collector.steps.len() >= max_steps;
    let match_attempts = collector.match_attempt + 1;

    unsafe { pcre2_match_context_free_8(match_context) };
    unsafe { pcre2_match_data_free_8(match_data) };
    unsafe { pcre2_code_free_8(code) };

    Ok((collector.steps, truncated, match_attempts))
}

pub fn build_offset_map(pattern: &str) -> Vec<PatternToken> {
    let Some(ast) = crate::explain::parse_ast(pattern) else {
        return Vec::new();
    };
    let mut tokens = Vec::new();
    collect_tokens(&ast, &mut tokens);
    tokens.sort_by_key(|t| t.start);
    tokens.dedup_by_key(|t| t.start);
    tokens
}

fn collect_tokens(ast: &regex_syntax::ast::Ast, tokens: &mut Vec<PatternToken>) {
    use crate::explain::formatter;
    use regex_syntax::ast::Ast;

    match ast {
        Ast::Empty(_) => {}
        Ast::Flags(f) => {
            tokens.push(PatternToken {
                start: f.span.start.offset,
                end: f.span.end.offset,
                description: formatter::format_flags_item(&f.flags),
            });
        }
        Ast::Literal(lit) => {
            tokens.push(PatternToken {
                start: lit.span.start.offset,
                end: lit.span.end.offset,
                description: formatter::format_literal(lit),
            });
        }
        Ast::Dot(span) => {
            tokens.push(PatternToken {
                start: span.start.offset,
                end: span.end.offset,
                description: "Any character".to_string(),
            });
        }
        Ast::Assertion(a) => {
            tokens.push(PatternToken {
                start: a.span.start.offset,
                end: a.span.end.offset,
                description: formatter::format_assertion(a),
            });
        }
        Ast::ClassUnicode(c) => {
            tokens.push(PatternToken {
                start: c.span.start.offset,
                end: c.span.end.offset,
                description: formatter::format_unicode_class(c),
            });
        }
        Ast::ClassPerl(c) => {
            tokens.push(PatternToken {
                start: c.span.start.offset,
                end: c.span.end.offset,
                description: formatter::format_perl_class(c),
            });
        }
        Ast::ClassBracketed(c) => {
            tokens.push(PatternToken {
                start: c.span.start.offset,
                end: c.span.end.offset,
                description: formatter::format_bracketed_class(c),
            });
        }
        Ast::Repetition(rep) => {
            collect_tokens(&rep.ast, tokens);
        }
        Ast::Group(group) => {
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

pub fn find_token_at_offset(offset_map: &[PatternToken], offset: usize) -> Option<usize> {
    let mut best: Option<(usize, usize)> = None;
    for (i, token) in offset_map.iter().enumerate() {
        if offset >= token.start && offset < token.end {
            return Some(i);
        }
        let dist = if offset < token.start {
            token.start - offset
        } else {
            offset - token.end
        };
        if best.map_or(true, |(_, d)| dist < d) {
            best = Some((i, dist));
        }
    }
    best.map(|(i, _)| i)
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
        assert!(!tokens.is_empty());
        assert_eq!(tokens[0].start, 0);
    }

    #[test]
    fn test_offset_map_groups() {
        let tokens = build_offset_map(r"(\d{3})-(\d{4})");
        assert!(!tokens.is_empty());
        let hyphen = tokens.iter().find(|t| t.description.contains('-'));
        assert!(hyphen.is_some());
    }

    #[test]
    fn test_offset_map_empty_pattern() {
        let tokens = build_offset_map("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn test_find_token_at_offset_basic() {
        let tokens = build_offset_map("abc");
        assert_eq!(find_token_at_offset(&tokens, 0), Some(0));
        assert_eq!(find_token_at_offset(&tokens, 1), Some(1));
        assert_eq!(find_token_at_offset(&tokens, 2), Some(2));
    }

    #[test]
    fn test_find_token_at_offset_empty() {
        let tokens: Vec<PatternToken> = Vec::new();
        assert_eq!(find_token_at_offset(&tokens, 0), None);
    }

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
        // a+ab against "aaab": greedy a+ takes all a's, then must backtrack
        // to give one back for the literal 'a' before 'b'
        let trace = debug_match("a+ab", "aaab", &flags, 10000, 0).unwrap();
        let has_backtrack = trace.steps.iter().any(|s| s.is_backtrack);
        assert!(has_backtrack, "should detect backtracking");
    }

    #[test]
    fn test_debug_match_step_limit() {
        let flags = EngineFlags::default();
        // Use a pattern/subject that generates many steps
        let trace = debug_match("a+ab", "aaaaaaaaaaaaaaaaaaaaab", &flags, 5, 0).unwrap();
        assert!(trace.truncated, "should truncate at step limit");
        assert_eq!(trace.steps.len(), 5);
    }

    #[test]
    fn test_debug_match_captures() {
        let flags = EngineFlags::default();
        let trace = debug_match("(a)(b)", "ab", &flags, 10000, 0).unwrap();
        let has_capture = trace
            .steps
            .iter()
            .any(|s| s.captures.iter().any(|c| c.is_some()));
        assert!(has_capture, "should capture groups during matching");
    }

    #[test]
    fn test_debug_match_start_offset() {
        let flags = EngineFlags::default();
        let trace = debug_match(r"\d+", "foo 123 bar 456", &flags, 10000, 8).unwrap();
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
        assert_eq!(trace.heatmap.len(), trace.offset_map.len());
        assert!(trace.heatmap.iter().any(|&c| c > 0));
    }
}
