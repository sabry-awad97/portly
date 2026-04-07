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
        /// Port number or PID to kill
        target: String,

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
}

impl Cli {
    pub fn parse_args() -> Self {
        Self::parse()
    }
}
