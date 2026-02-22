use super::{
    CaptureGroup, CompiledRegex, EngineError, EngineFlags, EngineKind, EngineResult, Match,
    RegexEngine,
};

pub struct FancyRegexEngine;

impl RegexEngine for FancyRegexEngine {
    fn kind(&self) -> EngineKind {
        EngineKind::FancyRegex
    }

    fn compile(&self, pattern: &str, flags: &EngineFlags) -> EngineResult<Box<dyn CompiledRegex>> {
        let full_pattern = flags.wrap_pattern(pattern);
        let re = fancy_regex::Regex::new(&full_pattern)
            .map_err(|e| EngineError::CompileError(e.to_string()))?;

        Ok(Box::new(FancyCompiledRegex { re }))
    }
}

struct FancyCompiledRegex {
    re: fancy_regex::Regex,
}

impl CompiledRegex for FancyCompiledRegex {
    fn find_matches(&self, text: &str) -> EngineResult<Vec<Match>> {
        let mut matches = Vec::new();

        for result in self.re.captures_iter(text) {
            let caps = result.map_err(|e| EngineError::MatchError(e.to_string()))?;
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
        let engine = FancyRegexEngine;
        let flags = EngineFlags::default();
        let compiled = engine.compile(r"\d+", &flags).unwrap();
        let matches = compiled.find_matches("abc 123 def 456").unwrap();
        assert_eq!(matches.len(), 2);
        assert_eq!(matches[0].text, "123");
    }

    #[test]
    fn test_lookahead() {
        let engine = FancyRegexEngine;
        let flags = EngineFlags::default();
        let compiled = engine.compile(r"\w+(?=@)", &flags).unwrap();
        let matches = compiled.find_matches("user@example.com").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "user");
    }

    #[test]
    fn test_named_captures() {
        let engine = FancyRegexEngine;
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
    fn test_lookbehind() {
        let engine = FancyRegexEngine;
        let flags = EngineFlags::default();
        let compiled = engine.compile(r"(?<=@)\w+", &flags).unwrap();
        let matches = compiled.find_matches("user@example.com").unwrap();
        assert_eq!(matches.len(), 1);
        assert_eq!(matches[0].text, "example");
    }
}
