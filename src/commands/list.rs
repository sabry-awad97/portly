use anyhow::Context;
use crate::{cli::Cli, config::Config, display, scanner::Scanner};

pub fn handle_list(scanner: &mut Scanner, cli: &Cli, _config: &Config) -> anyhow::Result<()> {
    let ports = scanner.scan(cli.all).context("Failed to scan ports")?;

    let display = display::Display::new(!cli.no_color, cli.json);
    display.show_ports(&ports);

    Ok(())
}
