use crate::scanner::Scanner;
use anyhow::Context;
use std::io::{self, Write};

pub fn handle_kill(
    scanner: &mut Scanner,
    targets: &[String],
    force: bool,
    json: bool,
    no_color: bool,
) -> anyhow::Result<()> {
    #[derive(Debug)]
    enum KillTarget {
        Port(u16, u32), // port, pid
        Pid(u32),
    }

    #[derive(serde::Serialize)]
    struct KillResult {
        target: String,
        pid: u32,
        success: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        error: Option<String>,
    }

    let mut resolved_targets = Vec::new();
    let mut results = Vec::new();

    // Resolve all targets first
    for target_str in targets {
        let target_num: u32 = target_str
            .parse()
            .context(format!("Invalid target: {}", target_str))?;

        // Try as port first (if <= 65535)
        if target_num <= 65535 {
            match scanner.get_port_details(target_num as u16) {
                Ok(port_info) => {
                    resolved_targets.push((
                        target_str.clone(),
                        KillTarget::Port(target_num as u16, port_info.pid),
                        port_info.process_name.clone(),
                    ));
                    continue;
                }
                Err(_) => {
                    // Not a port, try as PID
                }
            }
        }

        // Try as PID
        match scanner.get_process_info_by_pid(target_num) {
            Ok(process_info) => {
                resolved_targets.push((
                    target_str.clone(),
                    KillTarget::Pid(target_num),
                    process_info.name.clone(),
                ));
            }
            Err(_) => {
                if json {
                    results.push(KillResult {
                        target: target_str.clone(),
                        pid: target_num,
                        success: false,
                        error: Some(format!("Port or PID {} not found", target_num)),
                    });
                } else {
                    eprintln!("Error: Port or PID {} not found", target_num);
                }
            }
        }
    }

    if resolved_targets.is_empty() {
        if json {
            println!("{}", serde_json::to_string_pretty(&results)?);
        }
        return Ok(());
    }

    // Show what will be killed and ask for confirmation
    if !force && !json {
        println!("The following processes will be killed:");
        for (_target_str, kill_target, process_name) in &resolved_targets {
            match kill_target {
                KillTarget::Port(port, pid) => {
                    println!("  • :{} — {} (PID {})", port, process_name, pid);
                }
                KillTarget::Pid(pid) => {
                    println!("  • {} (PID {})", process_name, pid);
                }
            }
        }

        print!("\nKill {} process(es)? [y/N] ", resolved_targets.len());
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let answer = input.trim().to_lowercase();
        if answer != "y" && answer != "yes" {
            println!("Cancelled.");
            return Ok(());
        }
    }

    // Kill all processes
    for (target_str, kill_target, process_name) in resolved_targets {
        let pid = match kill_target {
            KillTarget::Port(_, pid) => pid,
            KillTarget::Pid(pid) => pid,
        };

        match scanner.kill_process(pid, force) {
            Ok(_) => {
                if json {
                    results.push(KillResult {
                        target: target_str.clone(),
                        pid,
                        success: true,
                        error: None,
                    });
                } else {
                    let success_msg = if no_color {
                        format!("✓ Killed {} (PID {})", process_name, pid)
                    } else {
                        use colored::Colorize;
                        format!("✓ Killed {} (PID {})", process_name, pid)
                            .green()
                            .to_string()
                    };
                    println!("{}", success_msg);
                }
            }
            Err(e) => {
                if json {
                    results.push(KillResult {
                        target: target_str.clone(),
                        pid,
                        success: false,
                        error: Some(e.to_string()),
                    });
                } else {
                    let error_msg = if no_color {
                        format!("✗ Failed to kill {} (PID {}): {}", process_name, pid, e)
                    } else {
                        use colored::Colorize;
                        format!("✗ Failed to kill {} (PID {}): {}", process_name, pid, e)
                            .red()
                            .to_string()
                    };
                    eprintln!("{}", error_msg);
                }
            }
        }
    }

    if json {
        println!("{}", serde_json::to_string_pretty(&results)?);
    }

    Ok(())
}
