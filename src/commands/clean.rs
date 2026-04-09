use crate::{process, scanner::Scanner};
use anyhow::Context;
use std::io::{self, Write};

pub fn handle_clean(
    scanner: &mut Scanner,
    execute: bool,
    json: bool,
    no_color: bool,
) -> anyhow::Result<()> {
    #[derive(serde::Serialize)]
    struct OrphanedProcess {
        port: Option<u16>,
        pid: u32,
        process_name: String,
        status: String,
    }

    #[derive(serde::Serialize)]
    struct CleanResult {
        found: usize,
        killed: usize,
        processes: Vec<OrphanedProcess>,
    }

    // Scan for all ports including system processes
    let ports = scanner.scan(true).context("Failed to scan ports")?;

    // Find orphaned and zombie processes
    let orphaned: Vec<_> = ports
        .iter()
        .filter(|p| {
            matches!(
                p.status,
                process::ProcessStatus::Orphaned | process::ProcessStatus::Zombie
            )
        })
        .collect();

    if orphaned.is_empty() {
        if json {
            let result = CleanResult {
                found: 0,
                killed: 0,
                processes: vec![],
            };
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!("No orphaned or zombie processes found.");
        }
        return Ok(());
    }

    // Prepare orphaned process list
    let orphaned_list: Vec<OrphanedProcess> = orphaned
        .iter()
        .map(|p| OrphanedProcess {
            port: Some(p.port),
            pid: p.pid,
            process_name: p.process_name.clone(),
            status: format!("{:?}", p.status),
        })
        .collect();

    if !execute {
        // Dry-run mode
        if json {
            let result = CleanResult {
                found: orphaned.len(),
                killed: 0,
                processes: orphaned_list,
            };
            println!("{}", serde_json::to_string_pretty(&result)?);
        } else {
            println!("Found {} orphaned/zombie process(es):", orphaned.len());
            for p in &orphaned {
                let status_str = if no_color {
                    format!("{:?}", p.status)
                } else {
                    use colored::Colorize;
                    match p.status {
                        process::ProcessStatus::Orphaned => "orphaned".yellow().to_string(),
                        process::ProcessStatus::Zombie => "zombie".red().to_string(),
                        _ => format!("{:?}", p.status),
                    }
                };
                println!(
                    "  • :{} — {} (PID {}) [{}]",
                    p.port, p.process_name, p.pid, status_str
                );
            }
            println!("\nRun with --execute to kill these processes.");
        }
        return Ok(());
    }

    // Execute mode - ask for confirmation
    if !json {
        println!("Found {} orphaned/zombie process(es):", orphaned.len());
        for p in &orphaned {
            let status_str = if no_color {
                format!("{:?}", p.status)
            } else {
                use colored::Colorize;
                match p.status {
                    process::ProcessStatus::Orphaned => "orphaned".yellow().to_string(),
                    process::ProcessStatus::Zombie => "zombie".red().to_string(),
                    _ => format!("{:?}", p.status),
                }
            };
            println!(
                "  • :{} — {} (PID {}) [{}]",
                p.port, p.process_name, p.pid, status_str
            );
        }

        print!("\nKill all {} process(es)? [y/N] ", orphaned.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let answer = input.trim().to_lowercase();
        if answer != "y" && answer != "yes" {
            println!("Cancelled.");
            return Ok(());
        }
    }

    // Kill all orphaned processes
    let mut killed_count = 0;
    for p in &orphaned {
        match scanner.kill_process(p.pid, false) {
            Ok(_) => {
                killed_count += 1;
                if !json {
                    let success_msg = if no_color {
                        format!("✓ Killed {} (PID {})", p.process_name, p.pid)
                    } else {
                        use colored::Colorize;
                        format!("✓ Killed {} (PID {})", p.process_name, p.pid)
                            .green()
                            .to_string()
                    };
                    println!("{}", success_msg);
                }
            }
            Err(e) => {
                if !json {
                    let error_msg = if no_color {
                        format!("✗ Failed to kill {} (PID {}): {}", p.process_name, p.pid, e)
                    } else {
                        use colored::Colorize;
                        format!("✗ Failed to kill {} (PID {}): {}", p.process_name, p.pid, e)
                            .red()
                            .to_string()
                    };
                    eprintln!("{}", error_msg);
                }
            }
        }
    }

    if json {
        let result = CleanResult {
            found: orphaned.len(),
            killed: killed_count,
            processes: orphaned_list,
        };
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("\nKilled {}/{} process(es).", killed_count, orphaned.len());
    }

    Ok(())
}
