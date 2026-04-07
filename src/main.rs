mod cli;
mod config;
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
            handle_list(&mut scanner, &cli, &config)?;
        }
        Some(Commands::Details { port, no_prompt }) => {
            handle_details(&mut scanner, port, no_prompt, cli.json)?;
        }
        Some(Commands::Kill { target, force }) => {
            handle_kill(&target, force)?;
        }
        Some(Commands::Clean { execute }) => {
            handle_clean(execute)?;
        }
        Some(Commands::Ps) => {
            handle_ps(&cli, &config)?;
        }
        Some(Commands::Watch { interval }) => {
            handle_watch(&mut scanner, interval, &cli, &config)?;
        }
    }

    Ok(())
}

fn handle_list(
    scanner: &mut Scanner,
    cli: &Cli,
    _config: &config::Config,
) -> anyhow::Result<()> {
    let ports = scanner.scan(cli.all).context("Failed to scan ports")?;

    let display = display::Display::new(!cli.no_color, cli.json);
    display.show_ports(&ports);

    Ok(())
}

fn handle_details(
    scanner: &mut Scanner,
    port: u16,
    _no_prompt: bool,
    json: bool,
) -> anyhow::Result<()> {
    let _port_info = scanner
        .get_port_details(port)
        .context(format!("Failed to get details for port {}", port))?;

    if json {
        println!("{{\"message\": \"Details view not yet implemented\"}}");
    } else {
        println!("Details view not yet implemented for port {}", port);
    }

    Ok(())
}

fn handle_kill(_target: &str, _force: bool) -> anyhow::Result<()> {
    println!("Kill command not yet implemented");
    Ok(())
}

fn handle_clean(_execute: bool) -> anyhow::Result<()> {
    println!("Clean command not yet implemented");
    Ok(())
}

fn handle_ps(_cli: &Cli, _config: &config::Config) -> anyhow::Result<()> {
    println!("Ps command not yet implemented");
    Ok(())
}

fn handle_watch(
    _scanner: &mut Scanner,
    _interval: u64,
    _cli: &Cli,
    _config: &config::Config,
) -> anyhow::Result<()> {
    println!("Watch command not yet implemented");
    Ok(())
}
