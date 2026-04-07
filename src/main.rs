mod cli;
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
            handle_list(&mut scanner, &cli, &config)?;
        }
        Some(Commands::Details { port, no_prompt }) => {
            handle_details(&mut scanner, port, no_prompt, cli.json, cli.no_color)?;
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
    no_prompt: bool,
    json: bool,
    no_color: bool,
) -> anyhow::Result<()> {
    let port_info = scanner
        .get_port_details(port)
        .context(format!("Failed to get details for port {}", port))?;

    let process_info = scanner
        .get_process_info_by_pid(port_info.pid)
        .context(format!("Failed to get process info for PID {}", port_info.pid))?;

    if json {
        // JSON output
        let json_output = serde_json::json!({
            "port": port_info.port,
            "pid": port_info.pid,
            "process_name": port_info.process_name,
            "status": port_info.status,
            "framework": port_info.framework,
            "project_name": port_info.project_name,
            "memory_kb": process_info.memory_kb,
            "command": process_info.command,
            "working_dir": process_info.working_dir,
        });
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else {
        // Detailed view
        details::show_port_details(&port_info, &process_info, scanner, !no_color)?;

        // Interactive kill prompt
        if !no_prompt {
            details::prompt_kill_process(port_info.pid, scanner)?;
        }
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
