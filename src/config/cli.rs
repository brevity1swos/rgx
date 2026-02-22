use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "rgx",
    version,
    about = "regex101, but in your terminal",
    long_about = "A terminal regex debugger with real-time matching, capture group highlighting, and plain-English explanations."
)]
pub struct Cli {
    /// Initial regex pattern
    #[arg(value_name = "PATTERN")]
    pub pattern: Option<String>,

    /// Engine to use: rust, fancy, or pcre2
    #[arg(short, long)]
    pub engine: Option<String>,

    /// Case-insensitive matching
    #[arg(short = 'i', long)]
    pub case_insensitive: bool,

    /// Multi-line mode
    #[arg(short = 'm', long)]
    pub multiline: bool,

    /// Dot matches newline
    #[arg(short = 's', long)]
    pub dotall: bool,

    /// Unicode mode
    #[arg(short = 'u', long)]
    pub unicode: Option<bool>,

    /// Extended mode (ignore whitespace)
    #[arg(short = 'x', long)]
    pub extended: bool,

    /// Initial replacement string
    #[arg(short = 'r', long)]
    pub replacement: Option<String>,
}
