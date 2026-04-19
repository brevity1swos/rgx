use super::{
    CaptureGroup, CompiledRegex, EngineError, EngineFlags, EngineKind, EngineResult, Match,
    RegexEngine,
};

/// Returns `true` when the linked PCRE2 is affected by CVE-2025-58050.
/// Affects PCRE2 10.45; upgrade to >= 10.46 to resolve.
///
/// Note: `pcre2::version()` returns hardcoded constants from `pcre2-sys`,
/// not the actual linked library version. We call `pcre2_config_8` directly
/// to get the real runtime version string (e.g. "10.45 2025-02-21").
pub fn is_pcre2_10_45() -> bool {
    runtime_pcre2_version() == Some((10, 45))
}

/// Queries the actual linked PCRE2 library for its version at runtime.
fn runtime_pcre2_version() -> Option<(u32, u32)> {
    use std::ffi::CStr;

    unsafe {
        // First call with null to get the required buffer size (in code units).
        let needed =
            pcre2_sys::pcre2_config_8(pcre2_sys::PCRE2_CONFIG_VERSION, std::ptr::null_mut());
        if needed <= 0 {
            return None;
        }
        let mut buf: Vec<u8> = vec![0u8; needed as usize];
        let rc = pcre2_sys::pcre2_config_8(
            pcre2_sys::PCRE2_CONFIG_VERSION,
            buf.as_mut_ptr() as *mut std::ffi::c_void,
        );
        if rc < 0 {
            return None;
        }
        let cstr = CStr::from_ptr(buf.as_ptr() as *const std::ffi::c_char);
        let s = cstr.to_str().ok()?;
        // Version string looks like "10.45 2025-02-21" — parse major.minor.
        let version_part = s.split_whitespace().next()?;
        let mut parts = version_part.split('.');
        let major = parts.next()?.parse::<u32>().ok()?;
        let minor = parts.next()?.parse::<u32>().ok()?;
        Some((major, minor))
    }
}

/// Returns `true` if `pattern` invokes the scan-substring verb (`(*scs:…)` or
/// `(*scan_substring:…)`) that is the documented trigger for CVE-2025-58050.
///
/// PCRE2 verb syntax is `(*NAME:…)` where NAME is immediately after `(*` with
/// no whitespace, so the match is precise enough for a local developer tool.
/// Verb names are case-insensitive in PCRE2.
fn uses_scs_verb(pattern: &str) -> bool {
    let lower = pattern.to_lowercase();
    lower.contains("(*scs:") || lower.contains("(*scan_substring:")
}

pub struct Pcre2Engine;

impl RegexEngine for Pcre2Engine {
    fn kind(&self) -> EngineKind {
        EngineKind::Pcre2
    }

    fn compile(&self, pattern: &str, flags: &EngineFlags) -> EngineResult<Box<dyn CompiledRegex>> {
        // CVE-2025-58050: block only patterns that invoke the scan-substring verb on
        // PCRE2 10.45. Disabling the entire engine would prevent all PCRE2 use for a
        // vulnerability that is specific to one verb's implementation. The status bar
        // already shows a persistent warning prompting the user to upgrade.
        if is_pcre2_10_45() && uses_scs_verb(pattern) {
            return Err(EngineError::CompileError(
                "Pattern blocked (CVE-2025-58050): (*scs:) / (*SCAN_SUBSTRING:) triggers \
                 a heap-buffer-overflow on the linked PCRE2 10.45. \
                 Upgrade to PCRE2 >= 10.46 to use this verb."
                    .to_string(),
            ));
        }

        let mut builder = pcre2::bytes::RegexBuilder::new();
        builder.utf(true);
        builder.ucp(flags.unicode);
        builder.caseless(flags.case_insensitive);
        builder.multi_line(flags.multi_line);
        builder.dotall(flags.dot_matches_newline);
        builder.extended(flags.extended);
        builder.jit_if_available(true);

        let re = builder
            .build(pattern)
            .map_err(|e| EngineError::CompileError(e.to_string()))?;

        Ok(Box::new(Pcre2CompiledRegex { re }))
    }
}

struct Pcre2CompiledRegex {
    re: pcre2::bytes::Regex,
}

impl CompiledRegex for Pcre2CompiledRegex {
    fn find_matches(&self, text: &str) -> EngineResult<Vec<Match>> {
        let mut matches = Vec::new();
        let bytes = text.as_bytes();

        let mut offset = 0;
        while offset <= bytes.len() {
            let caps = match self.re.captures(&bytes[offset..]) {
                Ok(Some(caps)) => caps,
                Ok(None) => break,
                Err(e) => return Err(EngineError::MatchError(e.to_string())),
            };

            let overall = caps.get(0).expect("capture group 0 must exist");
            if overall.start() == overall.end() && overall.start() == 0 && offset > 0 {
                offset += 1;
                continue;
            }

            let abs_start = offset + overall.start();
            let abs_end = offset + overall.end();

            let mut captures = Vec::new();
            let names = self.re.capture_names();
            for i in 1..caps.len() {
                if let Some(m) = caps.get(i) {
                    let cap_start = offset + m.start();
                    let cap_end = offset + m.end();
                    let name = names.get(i).and_then(Clone::clone);
                    captures.push(CaptureGroup {
                        index: i,
                        name,
                        start: cap_start,
                        end: cap_end,
                        text: text[cap_start..cap_end].to_string(),
                    });
                }
            }

            matches.push(Match {
                start: abs_start,
                end: abs_end,
                text: text[abs_start..abs_end].to_string(),
                captures,
            });

            if overall.start() == overall.end() {
                offset = abs_end + 1;
            } else {
                offset = abs_end;
            }
        }

        Ok(matches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_match() {
        let engine = Pcre2Engine;
        let flags = EngineFlags::default();
        let compiled = engine.compile(r"\d+", &flags).unwrap();
        let matches = compiled.find_matches("abc 123 def 456").unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].text, "123");
    }

    #[test]
    fn test_named_captures() {
        let engine = Pcre2Engine;
        let flags = EngineFlags::default();
        let compiled = engine
            .compile(r"(?P<user>\w+)@(?P<domain>\w+)", &flags)
            .unwrap();
        let matches = compiled.find_matches("user@example").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].captures.len(), 2);
        assert_eq!(matches[0].captures[0].name, Some("user".to_string()));
        assert_eq!(matches[0].captures[0].text, "user");
        assert_eq!(matches[0].captures[1].name, Some("domain".to_string()));
        assert_eq!(matches[0].captures[1].text, "example");
    }

    #[test]
    fn test_backreference() {
        let engine = Pcre2Engine;
        let flags = EngineFlags::default();
        let compiled = engine.compile(r"(\w+) \1", &flags).unwrap();
        let matches = compiled.find_matches("hello hello world").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "hello hello");
    }

    #[test]
    fn test_lookahead() {
        let engine = Pcre2Engine;
        let flags = EngineFlags::default();
        let compiled = engine.compile(r"\w+(?=@)", &flags).unwrap();
        let matches = compiled.find_matches("user@example.com").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "user");
    }

    #[test]
    fn test_uses_scs_verb_detection() {
        // CVE-2025-58050 trigger verb variants
        assert!(uses_scs_verb(r"(a)(b+)(*scs:(1)a(*ACCEPT))(\2)"));
        assert!(uses_scs_verb(r"(a)(*SCS:b)"));
        assert!(uses_scs_verb(r"(a)(*SCAN_SUBSTRING:b)"));
        assert!(uses_scs_verb(r"(a)(*scan_substring:b)"));
        // Unrelated patterns must not be blocked
        assert!(!uses_scs_verb(r"(\w+) \1"));
        assert!(!uses_scs_verb(r"(?<=@)\w+"));
        assert!(!uses_scs_verb(r"(*ACCEPT)"));
        assert!(!uses_scs_verb(r"(*FAIL)"));
    }

    // On PCRE2 != 10.45 the scs-verb check is the only guard; normal patterns
    // must still compile. On 10.45, scs patterns are blocked but all others work.
    #[test]
    fn test_non_scs_patterns_unaffected_by_cve_guard() {
        if !is_pcre2_10_45() {
            // All existing tests already verify this; just confirm the guard is off.
            let engine = Pcre2Engine;
            let flags = EngineFlags::default();
            assert!(engine.compile(r"\d+", &flags).is_ok());
            assert!(engine.compile(r"(?<=@)\w+", &flags).is_ok());
        }
    }
}
