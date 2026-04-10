use crate::{
    cli::Cli, config::Config, display::Display, progress::ProgressIndicator, scanner::Scanner,
};
use anyhow::Context;
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::time::Duration;

pub fn handle_watch(
    scanner: &mut Scanner,
    interval: u64,
    cli: &Cli,
    _config: &Config,
) -> anyhow::Result<()> {
    // Create Display instance
    let display = Display::new(!cli.no_color, cli.json);

    // Set up Ctrl+C handler
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .context("Failed to set Ctrl+C handler")?;

    // Display header
    println!("Watching for port changes...");
    println!("Press Ctrl+C to stop\n");

    // Initial scan with progress indicator
    let progress = ProgressIndicator::new("Performing initial scan...", cli.json, cli.quiet);

    let mut previous_ports: HashSet<u16> = scanner
        .scan(cli.all)
        .context("Failed to perform initial scan")?
        .into_iter()
        .map(|p| p.port)
        .collect();

    progress.finish();

    let sleep_duration = Duration::from_secs(interval);

    // Watch loop
    while running.load(Ordering::SeqCst) {
        thread::sleep(sleep_duration);

        // Scan for current ports (no progress indicator for periodic scans to keep it subtle)
        let current_ports_result = scanner.scan(cli.all);

        let current_ports = match current_ports_result {
            Ok(ports) => ports,
            Err(e) => {
                // Log error but continue watching
                eprintln!("Scan error: {}", e);
                continue;
            }
        };

        let current_set: HashSet<u16> = current_ports.iter().map(|p| p.port).collect();

        // Detect new ports
        for port in &current_set {
            if !previous_ports.contains(port)
                && let Some(info) = current_ports.iter().find(|p| p.port == *port)
            {
                display.show_watch_event_new(info);
            }
        }

        // Detect closed ports
        for port in &previous_ports {
            if !current_set.contains(port) {
                display.show_watch_event_closed(*port);
            }
        }

        previous_ports = current_set;
    }

    println!("\n\nStopped watching.\n");
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    #[test]
    fn test_watch_change_detection() {
        // Simulate previous ports
        let previous: HashSet<u16> = vec![3000, 5000, 8080].into_iter().collect();

        // Simulate current ports
        let current: HashSet<u16> = vec![3000, 5000, 9000].into_iter().collect();

        // Detect new ports
        let new_ports: Vec<u16> = current.difference(&previous).copied().collect();
        assert_eq!(new_ports, vec![9000]);

        // Detect closed ports
        let closed_ports: Vec<u16> = previous.difference(&current).copied().collect();
        assert_eq!(closed_ports, vec![8080]);
    }

    #[test]
    fn test_watch_no_changes() {
        let previous: HashSet<u16> = vec![3000, 5000].into_iter().collect();
        let current: HashSet<u16> = vec![3000, 5000].into_iter().collect();

        let new_ports: Vec<u16> = current.difference(&previous).copied().collect();
        assert!(new_ports.is_empty());

        let closed_ports: Vec<u16> = previous.difference(&current).copied().collect();
        assert!(closed_ports.is_empty());
    }
}
