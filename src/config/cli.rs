use clap::Parser;

#[derive(Parser, Debug)]
#[command(
    name = "rgx",
    version,
    about = "Terminal regex tester with real-time matching and multi-engine support",
    long_about = "Test and debug regular expressions without leaving your terminal. Supports 3 engines (Rust regex, fancy-regex, PCRE2), capture group highlighting, plain-English explanations, and replace mode. Useful for remote work, shell pipelines, and engine-specific testing."
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

    /// Read test string from file
    #[arg(short = 'f', long)]
    pub file: Option<String>,

    /// Test string (alternative to stdin or file)
    #[arg(short = 't', long)]
    pub text: Option<String>,

    /// Load workspace from file
    #[arg(short = 'l', long)]
    pub load: Option<String>,
}
