use regex::Regex;

use super::{
    CaptureGroup, CompiledRegex, EngineError, EngineFlags, EngineKind, EngineResult, Match,
    RegexEngine,
};

pub struct RustRegexEngine;

impl RegexEngine for RustRegexEngine {
    fn kind(&self) -> EngineKind {
        EngineKind::RustRegex
    }

    fn compile(&self, pattern: &str, flags: &EngineFlags) -> EngineResult<Box<dyn CompiledRegex>> {
        let full_pattern = flags.wrap_pattern(pattern);
        let re = Regex::new(&full_pattern).map_err(|e| EngineError::CompileError(e.to_string()))?;

        Ok(Box::new(RustCompiledRegex { re }))
    }
}

struct RustCompiledRegex {
    re: Regex,
}

impl CompiledRegex for RustCompiledRegex {
    fn find_matches(&self, text: &str) -> EngineResult<Vec<Match>> {
        let mut matches = Vec::new();

        for caps in self.re.captures_iter(text) {
            let overall = caps.get(0).unwrap();
            let mut captures = Vec::new();

            for (i, name) in self.re.capture_names().enumerate() {
                if i == 0 {
                    continue;
                }
                if let Some(m) = caps.get(i) {
                    captures.push(CaptureGroup {
                        index: i,
                        name: name.map(String::from),
                        start: m.start(),
                        end: m.end(),
                        text: m.as_str().to_string(),
                    });
                }
            }

            matches.push(Match {
                start: overall.start(),
                end: overall.end(),
                text: overall.as_str().to_string(),
                captures,
            });
        }

        Ok(matches)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_match() {
        let engine = RustRegexEngine;
        let flags = EngineFlags::default();
        let compiled = engine.compile(r"\d+", &flags).unwrap();
        let matches = compiled.find_matches("abc 123 def 456").unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].text, "123");
        assert_eq!(matches[1].text, "456");
    }

    #[test]
    fn test_capture_groups() {
        let engine = RustRegexEngine;
        let flags = EngineFlags::default();
        let compiled = engine.compile(r"(\w+)@(\w+)\.(\w+)", &flags).unwrap();
        let matches = compiled.find_matches("user@example.com").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].captures.len(), 3);
        assert_eq!(matches[0].captures[0].text, "user");
        assert_eq!(matches[0].captures[1].text, "example");
        assert_eq!(matches[0].captures[2].text, "com");
    }

    #[test]
    fn test_named_captures() {
        let engine = RustRegexEngine;
        let flags = EngineFlags::default();
        let compiled = engine
            .compile(r"(?P<name>\w+)@(?P<domain>\w+)", &flags)
            .unwrap();
        let matches = compiled.find_matches("user@example").unwrap();
        assert_eq!(matches[0].captures[0].name, Some("name".to_string()));
        assert_eq!(matches[0].captures[1].name, Some("domain".to_string()));
    }

    #[test]
    fn test_case_insensitive() {
        let engine = RustRegexEngine;
        let flags = EngineFlags {
            case_insensitive: true,
            ..Default::default()
        };
        let compiled = engine.compile(r"hello", &flags).unwrap();
        let matches = compiled.find_matches("Hello HELLO hello").unwrap();
        assert_eq!(matches.len(), 3);
    }

    #[test]
    fn test_invalid_pattern() {
        let engine = RustRegexEngine;
        let flags = EngineFlags::default();
        let result = engine.compile(r"(unclosed", &flags);
        assert!(result.is_err());
    }
}
