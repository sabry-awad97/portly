use crate::{
    platform::get_platform,
    process,
    scanner::{self, Scanner},
};
use anyhow::Context;

/// Process information for ps command
#[derive(Debug, Clone)]
pub struct PsProcess {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_kb: u64,
    pub project_name: Option<String>,
    pub framework: Option<String>,
    pub uptime: String,
    pub what: String,
}

pub fn handle_ps(
    scanner: &mut Scanner,
    show_all: bool,
    json: bool,
    no_color: bool,
    ascii: bool,
    config: &crate::config::Config,
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
            let display = crate::display::Display::new(false, false, config, false);
            let uptime = display.format_uptime(p.start_time);
            let what = display.format_command(&p.command, &p.name);
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
        let display = crate::display::Display::new(!no_color, false, config, ascii);
        display.show_ps_table(&ps_processes);
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
}
