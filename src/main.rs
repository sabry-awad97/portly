mod cli;
mod commands;
mod config;
mod details;
mod display;
mod docker;
mod error;
mod framework;
mod platform;
mod process;
mod scanner;

use anyhow::Context;
use cli::{Cli, Commands};
use platform::get_platform;
use scanner::Scanner;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse_args();

    // Load configuration
    let config = config::Config::load().context("Failed to load configuration")?;

    // Get platform implementation
    let platform = get_platform();

    // Create scanner
    let mut scanner = Scanner::new(platform);

    // Handle commands
    match cli.command {
        None | Some(Commands::List) => {
            commands::handle_list(&mut scanner, &cli, &config)?;
        }
        Some(Commands::Details { port, no_prompt }) => {
            commands::handle_details(&mut scanner, port, no_prompt, cli.json, cli.no_color)?;
        }
        Some(Commands::Kill { targets, force }) => {
            commands::handle_kill(&mut scanner, &targets, force, cli.json, cli.no_color)?;
        }
        Some(Commands::Clean { execute }) => {
            commands::handle_clean(&mut scanner, execute, cli.json, cli.no_color)?;
        }
        Some(Commands::Ps) => {
            commands::handle_ps(&mut scanner, cli.all, cli.json, cli.no_color)?;
        }
        Some(Commands::Watch { interval }) => {
            commands::handle_watch(&mut scanner, interval, &cli, &config)?;
        }
        Some(Commands::Config { action }) => {
            commands::handle_config(action)?;
        }
    }

    Ok(())
}
