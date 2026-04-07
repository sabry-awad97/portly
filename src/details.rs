use crate::error::Result;
use crate::process::{PortInfo, ProcessInfo};
use crate::scanner::Scanner;
use colored::*;
use humantime::format_duration;
use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::process::Command;
use std::time::SystemTime;

/// Display detailed information about a port
pub fn show_port_details(
    port_info: &PortInfo,
    process_info: &ProcessInfo,
    scanner: &Scanner,
    use_colors: bool,
) -> Result<()> {
    println!();
    println!("Port :{}", port_info.port);
    println!("──────────────────────");
    println!();

    // Process information
    println!("{:<16}{}", "Process", process_info.name);
    println!("{:<16}{}", "PID", process_info.pid);
    println!(
        "{:<16}{}",
        "Status",
        format_status(process_info.status, use_colors)
    );

    if let Some(ref framework) = port_info.framework {
        println!(
            "{:<16}{}",
            "Framework",
            format_framework(framework, use_colors)
        );
    }

    println!("{:<16}{}", "Memory", format_memory(process_info.memory_kb));

    if let Some(start_time) = process_info.start_time {
        println!("{:<16}{}", "Uptime", format_uptime(start_time));
    }

    println!();

    // Location information
    if process_info.working_dir.is_some() || port_info.project_name.is_some() {
        println!("Location");
        println!("──────────────────────");
        println!();

        if let Some(ref dir) = process_info.working_dir {
            println!("{:<16}{}", "Directory", dir);

            // Try to get git branch
            if let Some(branch) = get_git_branch(Path::new(dir)) {
                println!("{:<16}{}", "Branch", branch);
            }
        }

        if let Some(ref project) = port_info.project_name {
            println!("{:<16}{}", "Project", project);
        }

        println!();
    }

    // Command line
    if !process_info.command.is_empty() {
        println!("Command Line");
        println!("──────────────────────");
        println!();
        println!("{}", process_info.command);
        println!();
    }

    // Process tree
    match scanner.get_process_tree(process_info.pid) {
        Ok(tree) if !tree.is_empty() => {
            println!("Process Tree");
            println!("──────────────────────");
            println!();
            display_process_tree(&tree);
            println!();
        }
        _ => {}
    }

    Ok(())
}

/// Prompt user to kill the process
pub fn prompt_kill_process(pid: u32, scanner: &Scanner) -> Result<()> {
    // Check if we're in a TTY
    if !std::io::stdin().is_terminal() {
        return Ok(());
    }

    print!("Kill process {}? [y/N] ", pid);
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let answer = input.trim().to_lowercase();
    if answer == "y" || answer == "yes" {
        scanner.kill_process(pid, false)?;
        println!("Process {} killed successfully", pid);
    } else {
        println!("Process not killed");
    }

    Ok(())
}

/// Format process status with colors
fn format_status(status: crate::process::ProcessStatus, use_colors: bool) -> String {
    use crate::process::ProcessStatus;

    if !use_colors {
        return match status {
            ProcessStatus::Healthy => "● healthy".to_string(),
            ProcessStatus::Orphaned => "● orphaned".to_string(),
            ProcessStatus::Zombie => "● zombie".to_string(),
        };
    }

    match status {
        ProcessStatus::Healthy => format!("{} healthy", "●".green()),
        ProcessStatus::Orphaned => format!("{} orphaned", "●".yellow()),
        ProcessStatus::Zombie => format!("{} zombie", "●".red()),
    }
}

/// Format framework with colors
fn format_framework(framework: &str, use_colors: bool) -> String {
    if !use_colors {
        return framework.to_string();
    }

    match framework {
        "Next.js" | "Nuxt" | "Gatsby" => framework.cyan().to_string(),
        "Vite" | "Webpack" | "Parcel" => framework.bright_magenta().to_string(),
        "React" | "Vue" | "Angular" => framework.blue().to_string(),
        "Node.js" | "Express" => framework.green().to_string(),
        "Django" | "Flask" | "FastAPI" => framework.yellow().to_string(),
        "Rails" | "Ruby" => framework.red().to_string(),
        "Rust" | "Trunk" => framework.bright_red().to_string(),
        "Go" => framework.cyan().to_string(),
        "PostgreSQL" | "MySQL" => framework.blue().to_string(),
        "Redis" | "MongoDB" => framework.green().to_string(),
        "Docker" => framework.bright_blue().to_string(),
        _ => framework.normal().to_string(),
    }
}

/// Format memory in MB or GB
pub fn format_memory(memory_kb: u64) -> String {
    let memory_mb = memory_kb as f64 / 1024.0;

    if memory_mb >= 1024.0 {
        let memory_gb = memory_mb / 1024.0;
        format!("{:.1} GB", memory_gb)
    } else {
        format!("{:.1} MB", memory_mb)
    }
}

/// Format uptime as human-readable duration
fn format_uptime(start_time: SystemTime) -> String {
    let now = SystemTime::now();

    match now.duration_since(start_time) {
        Ok(duration) => format_duration(duration).to_string(),
        Err(_) => "unknown".to_string(),
    }
}

/// Get git branch from working directory
fn get_git_branch(cwd: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["-C", cwd.to_str()?, "rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .ok()?;

    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if !branch.is_empty() && branch != "HEAD" {
            return Some(branch);
        }
    }

    None
}

/// Display process tree with box-drawing characters
fn display_process_tree(tree: &[crate::process::ProcessNode]) {
    if tree.is_empty() {
        return;
    }

    // Tree is already in order from child to parent
    // Display from parent (last) to child (first)
    for (i, node) in tree.iter().rev().enumerate() {
        let indent = "│  ".repeat(i);
        let connector = if i == 0 { "├─" } else { "└─" };

        println!("{}{} {} ({})", indent, connector, node.name, node.pid);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_memory_mb() {
        assert_eq!(format_memory(1024), "1.0 MB");
        assert_eq!(format_memory(46234), "45.2 MB");
        assert_eq!(format_memory(512000), "500.0 MB");
    }

    #[test]
    fn test_format_memory_gb() {
        assert_eq!(format_memory(1048576), "1.0 GB");
        assert_eq!(format_memory(1572864), "1.5 GB");
        assert_eq!(format_memory(2097152), "2.0 GB");
    }

    #[test]
    fn test_format_uptime() {
        let one_hour_ago = SystemTime::now() - std::time::Duration::from_secs(3600);
        let uptime = format_uptime(one_hour_ago);
        assert!(uptime.contains("1h") || uptime.contains("hour"));
    }

    #[test]
    fn test_get_git_branch_not_a_repo() {
        let result = get_git_branch(Path::new("/nonexistent"));
        assert_eq!(result, None);
    }
}
