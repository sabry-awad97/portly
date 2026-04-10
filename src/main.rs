mod cli;
mod colors;
mod commands;
mod config;
mod details;
mod display;
mod docker;
mod error;
mod framework;
mod platform;
mod process;
mod progress;
mod scanner;
mod typo;

use anyhow::Context;
use cli::{Cli, Commands};
use colored::Colorize;
use platform::get_platform;
use scanner::Scanner;

#[tokio::main]
async fn main() {
    if let Err(e) = run().await {
        display_error(&e);
        std::process::exit(1);
    }
}

async fn run() -> anyhow::Result<()> {
    let cli = Cli::parse_args();

    // Load configuration
    let config = config::Config::load().context("Failed to load configuration")?;

    // Get platform implementation
    let platform = get_platform();

    // Create scanner with async Docker client
    let mut scanner = Scanner::new(platform).await;

    // Handle commands
    match cli.command {
        None | Some(Commands::List) => {
            commands::handle_list(&mut scanner, &cli, &config)?;
        }
        Some(Commands::Details { port, no_prompt }) => {
            commands::handle_details(
                &mut scanner,
                port,
                no_prompt,
                cli.json,
                cli.no_color,
                cli.ascii,
            )?;
        }
        Some(Commands::Kill { targets, force }) => {
            commands::handle_kill(&mut scanner, &targets, force, cli.json, cli.no_color)?;
        }
        Some(Commands::Clean { execute }) => {
            commands::handle_clean(&mut scanner, execute, cli.json, cli.no_color)?;
        }
        Some(Commands::Ps) => {
            commands::handle_ps(
                &mut scanner,
                cli.all,
                cli.json,
                cli.no_color,
                cli.ascii,
                cli.verbose,
                &config,
            )?;
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

fn display_error(error: &anyhow::Error) {
    // Check if colors should be disabled
    let use_color = !std::env::var("NO_COLOR").is_ok_and(|v| !v.is_empty());

    // Display error header and message
    if use_color {
        eprintln!("{}", "Error:".red().bold());
    } else {
        eprintln!("Error:");
    }
    eprintln!("  {error}");

    // Show suggestions if available
    if let Some(portly_err) = error.downcast_ref::<error::PortlyError>()
        && let Some(suggestion) = portly_err.suggestion()
    {
        eprintln!();
        if use_color {
            eprintln!("{}", "Suggestions:".yellow().bold());
        } else {
            eprintln!("Suggestions:");
        }
        for line in suggestion.lines() {
            eprintln!("  {line}");
        }
    }
}
