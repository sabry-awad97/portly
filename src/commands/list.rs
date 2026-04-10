use crate::{cli::Cli, config::Config, display, progress::ProgressIndicator, scanner::Scanner};
use anyhow::Context;

pub fn handle_list(scanner: &mut Scanner, cli: &Cli, config: &Config) -> anyhow::Result<()> {
    // Show progress indicator during scan
    let progress = ProgressIndicator::new("Scanning ports...", cli.json, cli.quiet);

    let ports = scanner.scan(cli.all).context("Failed to scan ports")?;

    // Finish progress before displaying results
    progress.finish();

    let display = display::Display::new(!cli.no_color, cli.json, config, cli.ascii, cli.verbose);

    // In verbose mode, fetch ProcessInfo for each port
    if cli.verbose && !cli.json {
        let mut process_infos = Vec::new();
        for port in &ports {
            if let Ok(proc_info) = scanner.get_process_info_by_pid(port.pid) {
                process_infos.push(proc_info);
            }
        }
        display.show_ports_verbose(&ports, &process_infos);
    } else {
        display.show_ports(&ports);
    }

    Ok(())
}
