use clap::{CommandFactory, Parser};
use clap_complete::{generate, Shell};

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
    #[arg(short = 'l', long, conflicts_with = "workspace")]
    pub load: Option<String>,

    /// Use a workspace file for save/load (creates if missing)
    #[arg(short = 'w', long, conflicts_with = "load")]
    pub workspace: Option<String>,

    /// Print matches to stdout and exit (non-interactive batch mode).
    /// Requires a pattern and input (stdin, --file, or --text).
    #[arg(short = 'p', long)]
    pub print: bool,

    /// After interactive session, print the final pattern to stdout instead of matches.
    /// Useful for: eval $(rgx -P)
    #[arg(short = 'P', long, conflicts_with = "print")]
    pub output_pattern: bool,

    /// Print a specific capture group instead of the full match (use with --print).
    /// Accepts a group number (1, 2, ...) or a named group.
    #[arg(short = 'g', long, requires = "print", conflicts_with = "count")]
    pub group: Option<String>,

    /// Print only the count of matches (use with --print).
    #[arg(short = 'c', long, requires = "print", conflicts_with = "group")]
    pub count: bool,

    /// Output matches as JSON (use with --print).
    #[arg(long, requires = "print")]
    pub json: bool,

    /// Colorize match output: auto (default), always, or never (use with --print).
    #[arg(long, default_value = "auto", requires = "print")]
    pub color: ColorMode,

    /// Use rounded border characters for panels.
    #[arg(long)]
    pub rounded: bool,

    /// Enable vim-style modal keybindings (Normal/Insert mode).
    #[arg(long)]
    pub vim: bool,

    /// Generate shell completions and exit.
    #[arg(long, value_name = "SHELL")]
    pub completions: Option<Shell>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, clap::ValueEnum)]
pub enum ColorMode {
    Auto,
    Always,
    Never,
}

impl Cli {
    pub fn print_completions(shell: Shell) {
        let mut cmd = Self::command();
        generate(shell, &mut cmd, "rgx", &mut std::io::stdout());
    }
}
