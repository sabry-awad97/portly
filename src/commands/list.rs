use crate::{cli::Cli, config::Config, display, progress::ProgressIndicator, scanner::Scanner};
use anyhow::Context;

pub fn handle_list(scanner: &mut Scanner, cli: &Cli, config: &Config) -> anyhow::Result<()> {
    // Show progress indicator during scan
    let progress = ProgressIndicator::new("Scanning ports...", cli.json, cli.quiet);

    let ports = scanner.scan(cli.all).context("Failed to scan ports")?;

    // Finish progress before displaying results
    progress.finish();

    let display = display::Display::new(!cli.no_color, cli.json, config, cli.ascii);
    display.show_ports(&ports);

    Ok(())
}
