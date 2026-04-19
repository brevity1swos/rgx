use regex_syntax::ast::{
    Assertion, AssertionKind, ClassBracketed, ClassPerl, ClassPerlKind, ClassSet, ClassSetItem,
    ClassUnicode, ClassUnicodeKind, FlagsItem, FlagsItemKind, Group, GroupKind, Literal,
    Repetition, RepetitionKind, RepetitionRange,
};

pub fn format_literal(lit: &Literal) -> String {
    let c = lit.c;
    if c.is_alphanumeric() {
        format!("Literal character '{c}'")
    } else {
        format!("Literal '{c}' (U+{:04X})", c as u32)
    }
}

pub fn format_assertion(assertion: &Assertion) -> String {
    match assertion.kind {
        AssertionKind::StartLine => "Start of line (^)".to_string(),
        AssertionKind::EndLine => "End of line ($)".to_string(),
        AssertionKind::StartText => "Start of text (\\A)".to_string(),
        AssertionKind::EndText => "End of text (\\z)".to_string(),
        AssertionKind::WordBoundary => "Word boundary (\\b)".to_string(),
        AssertionKind::NotWordBoundary => "Not a word boundary (\\B)".to_string(),
        _ => "Assertion".to_string(),
    }
}

pub fn format_perl_class(class: &ClassPerl) -> String {
    let negated = class.negated;
    match class.kind {
        ClassPerlKind::Digit => {
            if negated {
                "Non-digit character (\\D)".to_string()
            } else {
                "Digit character [0-9] (\\d)".to_string()
            }
        }
        ClassPerlKind::Space => {
            if negated {
                "Non-whitespace character (\\S)".to_string()
            } else {
                "Whitespace character (\\s)".to_string()
            }
        }
        ClassPerlKind::Word => {
            if negated {
                "Non-word character (\\W)".to_string()
            } else {
                "Word character [a-zA-Z0-9_] (\\w)".to_string()
            }
        }
    }
}

pub fn format_unicode_class(class: &ClassUnicode) -> String {
    let negated = if class.negated { "not " } else { "" };
    match &class.kind {
        ClassUnicodeKind::OneLetter(c) => {
            format!("Unicode property {negated}'{c}'")
        }
        ClassUnicodeKind::Named(name) => {
            format!("Unicode category {negated}'{name}'")
        }
        ClassUnicodeKind::NamedValue { name, value, .. } => {
            format!("Unicode property {negated}{name}={value}")
        }
    }
}

pub fn format_bracketed_class(class: &ClassBracketed) -> String {
    let negated = if class.negated { "not " } else { "" };
    let items = describe_class_set(&class.kind);
    format!("Character class: {negated}[{items}]")
}

fn describe_class_set(set: &ClassSet) -> String {
    match set {
        ClassSet::Item(item) => describe_class_set_item(item),
        ClassSet::BinaryOp(op) => {
            format!(
                "{} {:?} {}",
                describe_class_set(&op.lhs),
                op.kind,
                describe_class_set(&op.rhs)
            )
        }
    }
}

fn describe_class_set_item(item: &ClassSetItem) -> String {
    match item {
        ClassSetItem::Empty(_) => String::new(),
        ClassSetItem::Literal(lit) => lit.c.to_string(),
        ClassSetItem::Range(range) => {
            format!("{}-{}", range.start.c, range.end.c)
        }
        ClassSetItem::Ascii(ascii) => {
            let negated = if ascii.negated { "^" } else { "" };
            format!("{negated}{:?}", ascii.kind)
        }
        ClassSetItem::Unicode(u) => format_unicode_class(u),
        ClassSetItem::Perl(p) => format_perl_class(p),
        ClassSetItem::Bracketed(b) => format_bracketed_class(b),
        ClassSetItem::Union(union) => union
            .items
            .iter()
            .map(describe_class_set_item)
            .collect::<Vec<_>>()
            .join(", "),
    }
}

pub fn format_repetition(rep: &Repetition) -> String {
    let greedy = if rep.greedy { "" } else { " (lazy)" };
    match &rep.op.kind {
        RepetitionKind::ZeroOrOne => format!("Optional (0 or 1 time){greedy}"),
        RepetitionKind::ZeroOrMore => format!("Zero or more times{greedy}"),
        RepetitionKind::OneOrMore => format!("One or more times{greedy}"),
        RepetitionKind::Range(range) => match range {
            RepetitionRange::Exactly(n) => format!("Exactly {n} times"),
            RepetitionRange::AtLeast(n) => format!("At least {n} times{greedy}"),
            RepetitionRange::Bounded(min, max) => {
                format!("Between {min} and {max} times{greedy}")
            }
        },
    }
}

pub fn format_group(group: &Group) -> String {
    match &group.kind {
        GroupKind::CaptureIndex(idx) => {
            format!("Capture group #{}", idx)
        }
        GroupKind::CaptureName { name, .. } => {
            format!("Named capture group '{}'", name.name)
        }
        GroupKind::NonCapturing(_) => "Non-capturing group".to_string(),
    }
}

pub fn format_flags_item(flags: &regex_syntax::ast::Flags) -> String {
    let items: Vec<String> = flags.items.iter().map(format_single_flag).collect();
    format!("Set flags: {}", items.join(", "))
}

fn format_single_flag(item: &FlagsItem) -> String {
    match &item.kind {
        FlagsItemKind::Negation => "disable".to_string(),
        FlagsItemKind::Flag(flag) => {
            use regex_syntax::ast::Flag;
            match flag {
                Flag::CaseInsensitive => "case-insensitive (i)".to_string(),
                Flag::MultiLine => "multi-line (m)".to_string(),
                Flag::DotMatchesNewLine => "dot matches newline (s)".to_string(),
                Flag::SwapGreed => "swap greedy (U)".to_string(),
                Flag::Unicode => "unicode (u)".to_string(),
                Flag::CRLF => "CRLF mode (R)".to_string(),
                Flag::IgnoreWhitespace => "ignore whitespace (x)".to_string(),
            }
        }
    }
}
