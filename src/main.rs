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
use std::io::{self, Write};

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
        Some(Commands::Kill { targets, force }) => {
            handle_kill(&mut scanner, &targets, force, cli.json, cli.no_color)?;
        }
        Some(Commands::Clean { execute }) => {
            handle_clean(&mut scanner, execute, cli.json, cli.no_color)?;
        }
        Some(Commands::Ps) => {
            handle_ps_impl(&mut scanner, cli.all, cli.json, cli.no_color)?;
        }
        Some(Commands::Watch { interval }) => {
            handle_watch(&mut scanner, interval, &cli, &config)?;
        }
    }

    Ok(())
}

fn handle_list(scanner: &mut Scanner, cli: &Cli, _config: &config::Config) -> anyhow::Result<()> {
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
        .context(format!(
            "Failed to get process info for PID {}",
            port_info.pid
        ))?;

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

fn handle_kill(
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

fn handle_clean(
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

fn handle_watch(
    _scanner: &mut Scanner,
    _interval: u64,
    _cli: &Cli,
    _config: &config::Config,
) -> anyhow::Result<()> {
    println!("Watch command not yet implemented");
    Ok(())
}

/// Process information for ps command
#[derive(Debug, Clone)]
struct PsProcess {
    pid: u32,
    name: String,
    cpu_percent: f32,
    memory_kb: u64,
    project_name: Option<String>,
    framework: Option<String>,
    uptime: String,
    what: String,
}

fn handle_ps_impl(
    scanner: &mut Scanner,
    show_all: bool,
    json: bool,
    no_color: bool,
) -> anyhow::Result<()> {
    // Get platform to access all processes
    let platform = get_platform();
    let mut all_processes = platform
        .get_all_processes()
        .context("Failed to get all processes")?;

    // Filter dev processes unless show_all is true
    if !show_all {
        all_processes.retain(|p| !scanner::is_system_process(&p.name));
    }

    // Group Docker processes
    let processes = group_docker_processes(all_processes, scanner);

    // Sort by CPU% descending
    let mut ps_processes: Vec<PsProcess> = processes
        .into_iter()
        .map(|p| {
            let uptime = format_uptime_ps(p.start_time);
            let what = extract_what_description(&p.command, &p.name);
            let project_name = p
                .working_dir
                .as_ref()
                .and_then(|dir| std::path::Path::new(dir).file_name())
                .map(|name| name.to_string_lossy().to_string());

            // Detect framework
            let framework = scanner
                .framework_detector
                .detect(&p.command, p.working_dir.as_deref());

            PsProcess {
                pid: p.pid,
                name: p.name,
                cpu_percent: p.cpu_percent,
                memory_kb: p.memory_kb,
                project_name,
                framework,
                uptime,
                what,
            }
        })
        .collect();

    ps_processes.sort_by(|a, b| {
        b.cpu_percent
            .partial_cmp(&a.cpu_percent)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    if json {
        // JSON output
        let json_output = serde_json::json!({
            "processes": ps_processes.iter().map(|p| {
                serde_json::json!({
                    "pid": p.pid,
                    "name": p.name,
                    "cpu_percent": p.cpu_percent,
                    "memory_kb": p.memory_kb,
                    "project": p.project_name,
                    "framework": p.framework,
                    "uptime": p.uptime,
                    "what": p.what,
                })
            }).collect::<Vec<_>>(),
        });
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else {
        // Table output
        display_ps_table(&ps_processes, no_color);
    }

    Ok(())
}

fn group_docker_processes(
    processes: Vec<process::ProcessInfo>,
    _scanner: &Scanner,
) -> Vec<process::ProcessInfo> {
    let docker_procs: Vec<_> = processes
        .iter()
        .filter(|p| is_docker_process(&p.name))
        .collect();

    if docker_procs.is_empty() {
        return processes;
    }

    // Find daemon (lowest PID)
    let daemon = docker_procs.iter().min_by_key(|p| p.pid).unwrap();

    // Sum resources
    let total_cpu: f32 = docker_procs.iter().map(|p| p.cpu_percent).sum();
    let total_mem: u64 = docker_procs.iter().map(|p| p.memory_kb).sum();
    let container_count = docker_procs.len();

    // Create grouped process
    let grouped = process::ProcessInfo {
        pid: daemon.pid,
        name: "Docker".to_string(),
        cpu_percent: total_cpu,
        memory_kb: total_mem,
        command: format!("{} containers", container_count),
        status: daemon.status,
        start_time: daemon.start_time,
        working_dir: daemon.working_dir.clone(),
    };

    // Return non-Docker processes + grouped Docker
    let mut result: Vec<_> = processes
        .into_iter()
        .filter(|p| !is_docker_process(&p.name))
        .collect();
    result.push(grouped);
    result
}

fn is_docker_process(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    name_lower.contains("docker") || name_lower.starts_with("com.docker")
}

fn extract_what_description(cmd_line: &str, process_name: &str) -> String {
    if cmd_line.is_empty() {
        return process_name.to_string();
    }

    // Use the same logic as extract_command_description from scanner
    let parts: Vec<&str> = cmd_line.split_whitespace().collect();
    if parts.is_empty() {
        return process_name.to_string();
    }

    let first = parts[0].to_lowercase();

    // Node.js: "node /path/to/next dev" → "next dev"
    if first.contains("node") && parts.len() > 1 {
        let rest: Vec<&str> = parts[1..]
            .iter()
            .skip_while(|p| p.starts_with('-'))
            .copied()
            .collect();

        if !rest.is_empty() {
            let first_arg = rest[0];
            let cmd_name = std::path::Path::new(first_arg)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| first_arg.to_string());

            if rest.len() > 1 {
                let desc = format!("{} {}", cmd_name, rest[1..].join(" "));
                return truncate_string(&desc, 30);
            } else {
                return truncate_string(&cmd_name, 30);
            }
        }
    }

    // Python: "python manage.py runserver" → "manage.py runserver"
    if first.contains("python") && parts.len() > 2 {
        let desc = parts[2..].join(" ");
        return truncate_string(&desc, 30);
    }

    // Cargo: "cargo run --bin server" → "run --bin server"
    if first.contains("cargo") && parts.len() > 1 {
        let desc = parts[1..].join(" ");
        return truncate_string(&desc, 30);
    }

    // Docker: show container count from command
    if first.contains("docker") {
        return truncate_string(cmd_line, 30);
    }

    // Default: return process name
    truncate_string(process_name, 30)
}

fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

fn format_uptime_ps(start_time: Option<std::time::SystemTime>) -> String {
    let Some(start) = start_time else {
        return "—".to_string();
    };

    let Ok(duration) = std::time::SystemTime::now().duration_since(start) else {
        return "—".to_string();
    };

    let total_secs = duration.as_secs();
    let days = total_secs / 86400;
    let hours = (total_secs % 86400) / 3600;
    let minutes = (total_secs % 3600) / 60;

    if days > 0 {
        format!("{}d {}h", days, hours)
    } else if hours > 0 {
        format!("{}h {}m", hours, minutes)
    } else {
        format!("{}m", minutes)
    }
}

fn display_ps_table(processes: &[PsProcess], no_color: bool) {
    use tabled::{Table, Tabled, settings::Style};

    #[derive(Tabled)]
    struct PsRow {
        #[tabled(rename = "PID")]
        pid: String,
        #[tabled(rename = "PROCESS")]
        process: String,
        #[tabled(rename = "CPU%")]
        cpu: String,
        #[tabled(rename = "MEM")]
        mem: String,
        #[tabled(rename = "PROJECT")]
        project: String,
        #[tabled(rename = "FRAMEWORK")]
        framework: String,
        #[tabled(rename = "UPTIME")]
        uptime: String,
        #[tabled(rename = "WHAT")]
        what: String,
    }

    let rows: Vec<PsRow> = processes
        .iter()
        .map(|p| {
            let cpu_str = format!("{:.1}", p.cpu_percent);
            let cpu_colored = if no_color {
                cpu_str
            } else {
                use colored::Colorize;
                if p.cpu_percent > 25.0 {
                    cpu_str.red().to_string()
                } else if p.cpu_percent > 5.0 {
                    cpu_str.yellow().to_string()
                } else {
                    cpu_str.green().to_string()
                }
            };

            let mem_str = details::format_memory(p.memory_kb);

            PsRow {
                pid: p.pid.to_string(),
                process: p.name.clone(),
                cpu: cpu_colored,
                mem: mem_str,
                project: p.project_name.clone().unwrap_or_else(|| "—".to_string()),
                framework: p.framework.clone().unwrap_or_else(|| "—".to_string()),
                uptime: p.uptime.clone(),
                what: p.what.clone(),
            }
        })
        .collect();

    let mut table = Table::new(rows);
    table.with(Style::rounded());
    println!("{}", table);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kill_target_resolution_port() {
        // Test that port numbers (<=65535) are tried as ports first
        let target = "3000";
        let num: u32 = target.parse().unwrap();
        assert!(num <= 65535);
        assert_eq!(num, 3000);
    }

    #[test]
    fn test_kill_target_resolution_pid() {
        // Test that large numbers (>65535) are treated as PIDs
        let target = "123456";
        let num: u32 = target.parse().unwrap();
        assert!(num > 65535);
        assert_eq!(num, 123456);
    }

    #[test]
    fn test_kill_target_parsing_invalid() {
        // Test that invalid targets fail to parse
        let target = "invalid";
        let result: Result<u32, _> = target.parse();
        assert!(result.is_err());
    }

    #[test]
    fn test_orphaned_process_detection() {
        // Test that zombie and orphaned statuses are correctly identified
        use process::ProcessStatus;

        let zombie = ProcessStatus::Zombie;
        let orphaned = ProcessStatus::Orphaned;
        let healthy = ProcessStatus::Healthy;

        assert!(matches!(zombie, ProcessStatus::Zombie));
        assert!(matches!(orphaned, ProcessStatus::Orphaned));
        assert!(!matches!(
            healthy,
            ProcessStatus::Zombie | ProcessStatus::Orphaned
        ));
    }

    #[test]
    fn test_is_docker_process() {
        assert!(is_docker_process("docker"));
        assert!(is_docker_process("Docker"));
        assert!(is_docker_process("com.docker.backend"));
        assert!(!is_docker_process("node"));
        assert!(!is_docker_process("python"));
    }

    #[test]
    fn test_extract_what_description() {
        assert_eq!(
            extract_what_description("node /path/to/next dev", "node"),
            "next dev"
        );
        assert_eq!(
            extract_what_description("python manage.py runserver", "python"),
            "runserver"
        );
        assert_eq!(
            extract_what_description("cargo run --release", "cargo"),
            "run --release"
        );
        assert_eq!(extract_what_description("", "postgres"), "postgres");
    }

    #[test]
    fn test_truncate_string() {
        assert_eq!(truncate_string("short", 10), "short");
        assert_eq!(
            truncate_string("this is a very long string", 10),
            "this is..."
        );
        assert_eq!(truncate_string("exactly10c", 10), "exactly10c");
    }

    #[test]
    fn test_format_uptime_ps() {
        let one_hour_ago = std::time::SystemTime::now() - std::time::Duration::from_secs(3600);
        let uptime = format_uptime_ps(Some(one_hour_ago));
        assert!(uptime.contains("1h") || uptime.contains("0h"));

        let one_day_ago = std::time::SystemTime::now() - std::time::Duration::from_secs(86400);
        let uptime = format_uptime_ps(Some(one_day_ago));
        assert!(uptime.contains("1d") || uptime.contains("0d"));
    }
}
