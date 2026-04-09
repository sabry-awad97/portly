use crate::{cli::Cli, config::Config, process, scanner::Scanner};
use anyhow::Context;
use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

pub fn handle_watch(
    scanner: &mut Scanner,
    interval: u64,
    cli: &Cli,
    _config: &Config,
) -> anyhow::Result<()> {
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

    // Initial scan
    let mut previous_ports: HashSet<u16> = scanner
        .scan(cli.all)
        .context("Failed to perform initial scan")?
        .into_iter()
        .map(|p| p.port)
        .collect();

    let sleep_duration = Duration::from_secs(interval);

    // Watch loop
    while running.load(Ordering::SeqCst) {
        thread::sleep(sleep_duration);

        // Scan for current ports
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
                display_watch_event_new(info, cli.no_color);
            }
        }

        // Detect closed ports
        for port in &previous_ports {
            if !current_set.contains(port) {
                display_watch_event_closed(*port, cli.no_color);
            }
        }

        previous_ports = current_set;
    }

    println!("\n\nStopped watching.\n");
    Ok(())
}

fn display_watch_event_new(port_info: &process::PortInfo, no_color: bool) {
    let timestamp = chrono::Local::now().format("%H:%M:%S");
    let event_marker = if no_color {
        "▲ NEW   "
    } else {
        use colored::Colorize;
        &format!("{}", "▲ NEW   ".green())
    };

    let framework_str = port_info
        .framework
        .as_ref()
        .map(|f| format!(" {}", f))
        .unwrap_or_default();

    let project_str = port_info
        .project_name
        .as_ref()
        .map(|p| format!(" [{}]", p))
        .unwrap_or_default();

    println!(
        "{} {} :{} ← {}{}{}",
        timestamp, event_marker, port_info.port, port_info.process_name, project_str, framework_str
    );
}

fn display_watch_event_closed(port: u16, no_color: bool) {
    let timestamp = chrono::Local::now().format("%H:%M:%S");
    let event_marker = if no_color {
        "▼ CLOSED"
    } else {
        use colored::Colorize;
        &format!("{}", "▼ CLOSED".red())
    };

    println!("{} {} :{}", timestamp, event_marker, port);
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
