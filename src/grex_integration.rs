//! Pure wrapper around the `grex` crate for regex generation from example strings.

use grex::RegExpBuilder;

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
