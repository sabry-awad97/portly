use crate::error::Result;
use crate::platform::Platform;
use crate::process::PortInfo;
use std::path::Path;

/// Port scanner orchestrator
pub struct Scanner {
    platform: Box<dyn Platform>,
}

impl Scanner {
    pub fn new(platform: Box<dyn Platform>) -> Self {
        Self { platform }
    }

    /// Scan for all listening ports
    pub fn scan(&self, show_all: bool) -> Result<Vec<PortInfo>> {
        // Get raw port info from platform
        let raw_ports = self.platform.get_listening_ports()?;

        let mut ports = Vec::new();

        for raw_port in raw_ports {
            // Get process info
            let process_info = match self.platform.get_process_info(raw_port.pid) {
                Ok(info) => info,
                Err(_) => continue, // Skip if we can't get process info
            };

            // Filter system processes unless show_all is true
            if !show_all && is_system_process(&process_info.name) {
                continue;
            }

            // Extract smart command description
            let process_name = extract_command_description(&process_info.command, &process_info.name);

            // Detect framework (stub for now, will be implemented in Issue #4)
            let framework = None;

            // Extract project name from working directory
            let project_name = process_info
                .working_dir
                .as_ref()
                .and_then(|dir| Path::new(dir).file_name())
                .map(|name| name.to_string_lossy().to_string());

            ports.push(PortInfo {
                port: raw_port.port,
                pid: raw_port.pid,
                process_name,
                status: process_info.status,
                framework,
                project_name,
            });
        }

        // Sort by port number
        ports.sort_by_key(|p| p.port);

        Ok(ports)
    }

    /// Get detailed information for a specific port
    pub fn get_port_details(&self, port: u16) -> Result<PortInfo> {
        let ports = self.scan(true)?; // Show all when looking for specific port
        ports
            .into_iter()
            .find(|p| p.port == port)
            .ok_or_else(|| crate::error::PortlyError::PortNotFound(port))
    }
}

/// Check if a process is a system process that should be filtered
fn is_system_process(name: &str) -> bool {
    let name_lower = name.to_lowercase();

    // System processes
    const SYSTEM_PROCESSES: &[&str] = &[
        "svchost",
        "csrss",
        "lsass",
        "services",
        "explorer",
        "dwm",
        "searchindexer",
        "taskhostw",
        "runtimebroker",
        "system",
        "registry",
        "smss",
        "wininit",
        "winlogon",
    ];

    // User applications to filter
    const FILTERED_APPS: &[&str] = &[
        "spotify",
        "chrome",
        "firefox",
        "slack",
        "discord",
        "code", // VS Code
        "teams",
        "zoom",
        "skype",
        "msedge",
        "brave",
    ];

    SYSTEM_PROCESSES.iter().any(|&sys| name_lower.contains(sys))
        || FILTERED_APPS.iter().any(|&app| name_lower.contains(app))
}

/// Extract meaningful command description from full command line
fn extract_command_description(cmd_line: &str, process_name: &str) -> String {
    if cmd_line.is_empty() {
        return process_name.to_string();
    }

    let parts: Vec<&str> = cmd_line.split_whitespace().collect();
    if parts.is_empty() {
        return process_name.to_string();
    }

    let first = parts[0].to_lowercase();

    // Node.js: "node /path/to/next dev" → "next dev"
    if first.contains("node") && parts.len() > 1 {
        // Skip the node executable and flags, find the actual command
        let rest: Vec<&str> = parts[1..]
            .iter()
            .skip_while(|p| p.starts_with('-'))
            .copied()
            .collect();

        if !rest.is_empty() {
            // First non-flag argument is usually the script/command
            let first_arg = rest[0];
            
            // Extract basename if it's a path
            let cmd_name = Path::new(first_arg)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| first_arg.to_string());
            
            // Include remaining arguments
            if rest.len() > 1 {
                return format!("{} {}", cmd_name, rest[1..].join(" "));
            } else {
                return cmd_name;
            }
        }
    }

    // Python: "python manage.py runserver" → "runserver"
    if first.contains("python") && parts.len() > 2 {
        return parts[2..].join(" ");
    }

    // Cargo: "cargo run --bin server" → "cargo run --bin server"
    if first.contains("cargo") {
        return parts.join(" ");
    }

    // Go: "go run main.go" → "go run"
    if first.contains("go") && parts.len() > 1 {
        return parts[..2].join(" ");
    }

    // Dotnet: "dotnet run" → "dotnet run"
    if first.contains("dotnet") && parts.len() > 1 {
        return parts[..2].join(" ");
    }

    // Docker: show just "docker"
    if first.contains("docker") {
        return "docker".to_string();
    }

    // Default: return process name
    process_name.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_command_description_node() {
        assert_eq!(
            extract_command_description("node /usr/bin/next dev", "node"),
            "next dev"
        );
        assert_eq!(
            extract_command_description("node C:\\path\\to\\vite --port 3000", "node"),
            "vite --port 3000"
        );
    }

    #[test]
    fn test_extract_command_description_python() {
        assert_eq!(
            extract_command_description("python manage.py runserver", "python"),
            "runserver"
        );
        assert_eq!(
            extract_command_description("python3 manage.py migrate", "python3"),
            "migrate"
        );
    }

    #[test]
    fn test_extract_command_description_cargo() {
        assert_eq!(
            extract_command_description("cargo run --bin server", "cargo"),
            "cargo run --bin server"
        );
    }

    #[test]
    fn test_extract_command_description_fallback() {
        assert_eq!(
            extract_command_description("", "postgres"),
            "postgres"
        );
        assert_eq!(
            extract_command_description("unknown command", "unknown"),
            "unknown"
        );
    }

    #[test]
    fn test_is_system_process() {
        assert!(is_system_process("svchost.exe"));
        assert!(is_system_process("explorer.exe"));
        assert!(is_system_process("Spotify.exe"));
        assert!(is_system_process("chrome.exe"));
        assert!(!is_system_process("node.exe"));
        assert!(!is_system_process("python.exe"));
        assert!(!is_system_process("cargo.exe"));
    }
}
