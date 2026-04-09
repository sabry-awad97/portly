use crate::{
    details,
    platform::get_platform,
    process,
    scanner::{self, Scanner},
};
use anyhow::Context;

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

pub fn handle_ps(
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
    let daemon = docker_procs
        .iter()
        .min_by_key(|p| p.pid)
        .expect("docker_procs is non-empty (checked above)");

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
    let days = total_secs.saturating_div(86400);
    let hours = (total_secs % 86400).saturating_div(3600);
    let minutes = (total_secs % 3600).saturating_div(60);

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
