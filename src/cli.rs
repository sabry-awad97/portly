use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "portly")]
#[command(author, version, about, long_about = None)]
#[command(
    about = "Modern CLI tool for managing local development ports",
    long_about = "Portly helps you understand what's running on your ports with beautiful tables, \
                  framework detection, and interactive process management."
)]
pub struct Cli {
    /// Enable JSON output
    #[arg(long, global = true)]
    pub json: bool,

    /// Show all processes including system apps
    #[arg(long, global = true)]
    pub all: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Suppress progress indicators
    #[arg(long, global = true)]
    pub quiet: bool,

    /// Use ASCII-only output (no Unicode symbols)
    #[arg(long, global = true)]
    pub ascii: bool,

    /// Show detailed output with full descriptions (no truncation)
    #[arg(short = 'v', long, global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all listening ports (default command)
    List,

    /// Show detailed information about a specific port
    #[command(name = "details", alias = "info")]
    Details {
        /// Port number to inspect
        port: u16,

        /// Skip interactive kill prompt
        #[arg(long)]
        no_prompt: bool,
    },

    /// Kill process by port or PID
    Kill {
        /// Port numbers or PIDs to kill (e.g., 3000, 5000, 12345)
        #[arg(required = true)]
        targets: Vec<String>,

        /// Force kill without confirmation
        #[arg(short, long)]
        force: bool,
    },

    /// Find and kill orphaned processes
    Clean {
        /// Actually kill processes (dry-run by default)
        #[arg(long)]
        execute: bool,
    },

    /// List all dev processes (not just port-bound)
    Ps,

    /// Watch for port changes in real-time
    Watch {
        /// Update interval in seconds
        #[arg(short, long, default_value = "2")]
        interval: u64,
    },

    /// Manage configuration
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Initialize default config file
    Init,

    /// Show config file path
    Path,

    /// Reset config to defaults
    Reset,
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_accepts_ascii_flag() {
        let cli = Cli::parse_from(["portly", "--ascii", "list"]);
        assert!(cli.ascii);
    }

    #[test]
    fn test_cli_accepts_verbose_flag() {
        let cli = Cli::parse_from(["portly", "--verbose", "list"]);
        assert!(cli.verbose);
    }

    #[test]
    fn test_cli_accepts_verbose_short_flag() {
        let cli = Cli::parse_from(["portly", "-v", "list"]);
        assert!(cli.verbose);
    }
}
