use clap::{Args, CommandFactory, Parser, Subcommand};
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

    /// Run test assertions from workspace file(s) and exit.
    /// Expects TOML files with \[\[tests\]\] sections.
    #[arg(long, num_args = 1.., value_name = "FILE")]
    pub test: Option<Vec<String>>,

    /// Use rounded border characters for panels.
    #[arg(long)]
    pub rounded: bool,

    /// Enable vim-style modal keybindings (Normal/Insert mode).
    #[arg(long)]
    pub vim: bool,

    /// Generate shell completions and exit.
    #[arg(long, value_name = "SHELL")]
    pub completions: Option<Shell>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Live-filter stdin or a file through a regex (grep-like, with a TUI).
    Filter(FilterArgs),
}

#[derive(Args, Debug, Clone)]
pub struct FilterArgs {
    /// Regex pattern to filter with. If omitted, the TUI starts with an empty pattern.
    #[arg(value_name = "PATTERN")]
    pub pattern: Option<String>,

    /// Read input from a file instead of stdin.
    #[arg(short = 'f', long)]
    pub file: Option<std::path::PathBuf>,

    /// Invert the match — emit lines that do NOT match (like `grep -v`).
    #[arg(short = 'v', long)]
    pub invert: bool,

    /// Emit only the count of matching lines (non-interactive).
    #[arg(short = 'c', long)]
    pub count: bool,

    /// Prefix each emitted line with its line number (non-interactive).
    #[arg(short = 'n', long = "line-number")]
    pub line_number: bool,

    /// Case-insensitive matching (equivalent to Alt+i inside the TUI).
    #[arg(short = 'i', long)]
    pub case_insensitive: bool,

    /// Cap input at N lines to prevent OOM on multi-GB piped streams.
    /// Defaults to 100000. Pass a larger value (or 0 for no cap — see below)
    /// if you know your input fits in memory.
    #[arg(long, default_value_t = 100_000, value_name = "N")]
    pub max_lines: usize,

    /// Extract a field from JSON lines using a dotted path before matching.
    /// Example: --json '.msg' filters the `msg` field of each JSONL record.
    /// Non-string values and parse failures skip that line silently. The raw
    /// JSON line is still what gets emitted when matched.
    #[arg(long, value_name = "PATH")]
    pub json: Option<String>,
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
