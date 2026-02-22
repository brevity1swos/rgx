use super::{
    CaptureGroup, CompiledRegex, EngineError, EngineFlags, EngineKind, EngineResult, Match,
    RegexEngine,
};

pub struct Pcre2Engine;

impl RegexEngine for Pcre2Engine {
    fn kind(&self) -> EngineKind {
        EngineKind::Pcre2
    }

    fn compile(&self, pattern: &str, flags: &EngineFlags) -> EngineResult<Box<dyn CompiledRegex>> {
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

            let overall = caps.get(0).unwrap();
            if overall.start() == overall.end() && overall.start() == 0 && offset > 0 {
                offset += 1;
                continue;
            }

            let abs_start = offset + overall.start();
            let abs_end = offset + overall.end();

            let mut captures = Vec::new();
            for i in 1..caps.len() {
                if let Some(m) = caps.get(i) {
                    let cap_start = offset + m.start();
                    let cap_end = offset + m.end();
                    captures.push(CaptureGroup {
                        index: i,
                        name: None,
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
                offset += abs_end + 1;
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
}
