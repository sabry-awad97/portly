use crate::docker::DockerClient;
use crate::error::Result;
use crate::framework::FrameworkDetector;
use crate::platform::Platform;
use crate::process::PortInfo;
use std::path::Path;

/// Port scanner orchestrator.
///
/// Coordinates platform-specific port scanning with framework detection
/// and Docker integration.
///
/// # Examples
///
/// ```no_run
/// use portly::scanner::Scanner;
/// use portly::platform::get_platform;
///
/// let platform = get_platform();
/// let mut scanner = Scanner::new(platform);
/// let ports = scanner.scan(false)?;
/// # Ok::<(), portly::error::PortlyError>(())
/// ```
pub struct Scanner {
    platform: Box<dyn Platform>,
    pub framework_detector: FrameworkDetector,
    docker_client: DockerClient,
}

impl Scanner {
    /// Create a new scanner with the given platform implementation.
    ///
    /// # Arguments
    ///
    /// * `platform` - Platform-specific implementation for port scanning
    pub fn new(platform: Box<dyn Platform>) -> Self {
        Self {
            platform,
            framework_detector: FrameworkDetector::new(),
            docker_client: DockerClient::new(),
        }
    }

    /// Create a new async scanner with Bollard-based Docker client
    ///
    /// # Arguments
    ///
    /// * `platform` - Platform-specific implementation for port scanning
    ///
    /// # Errors
    ///
    /// Returns error if Docker connection fails (gracefully falls back to empty client)
    pub async fn new_async(platform: Box<dyn Platform>) -> Self {
        // Try to create async Docker client, fallback to sync if it fails
        let docker_client = DockerClient::new_async().await.unwrap_or_else(|_| DockerClient::new());
        
        Self {
            platform,
            framework_detector: FrameworkDetector::new(),
            docker_client,
        }
    }

    /// Scan for all listening ports.
    ///
    /// # Arguments
    ///
    /// * `show_all` - If true, include system processes; if false, filter them out
    ///
    /// # Returns
    ///
    /// Vector of enriched port information with process details and framework detection
    ///
    /// # Errors
    ///
    /// Returns error if port scanning fails or process information cannot be retrieved
    #[must_use = "scan results should be processed"]
    pub fn scan(&mut self, show_all: bool) -> Result<Vec<PortInfo>> {
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
            let process_name =
                extract_command_description(&process_info.command, &process_info.name);

            // Check if this port is mapped to a Docker container
            let (framework, project_name) =
                if let Some(container) = self.docker_client.get_container_info(raw_port.port) {
                    // Docker container found
                    let framework = DockerClient::detect_framework_from_image(&container.image);
                    let project_name = Some(container.name.clone());
                    (framework, project_name)
                } else {
                    // Not a Docker container, use regular detection
                    let framework = self
                        .framework_detector
                        .detect(&process_info.command, process_info.working_dir.as_deref());

                    let project_name = process_info
                        .working_dir
                        .as_ref()
                        .and_then(|dir| Path::new(dir).file_name())
                        .map(|name| name.to_string_lossy().to_string());

                    (framework, project_name)
                };

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

    /// Get detailed information for a specific port.
    ///
    /// # Arguments
    ///
    /// * `port` - Port number to inspect
    ///
    /// # Returns
    ///
    /// Detailed port information including process details
    ///
    /// # Errors
    ///
    /// Returns `PortlyError::PortNotFound` if the port is not in use
    #[must_use = "port details should be processed"]
    pub fn get_port_details(&mut self, port: u16) -> Result<PortInfo> {
        let ports = self.scan(true)?; // Show all when looking for specific port
        ports
            .into_iter()
            .find(|p| p.port == port)
            .ok_or(crate::error::PortlyError::PortNotFound {
                port,
                suggestion: Some(
                    "• Run 'portly list' to see all listening ports\n\
                     • Check if the process is still running\n\
                     • Try 'portly ps' to see all dev processes"
                        .to_string(),
                ),
            })
    }

    /// Get process information by PID
    pub fn get_process_info_by_pid(&self, pid: u32) -> Result<crate::process::ProcessInfo> {
        self.platform.get_process_info(pid)
    }

    /// Get process tree for a PID
    pub fn get_process_tree(&self, pid: u32) -> Result<Vec<crate::process::ProcessNode>> {
        self.platform.get_process_tree(pid)
    }

    /// Kill a process by PID
    pub fn kill_process(&self, pid: u32, force: bool) -> Result<()> {
        self.platform.kill_process(pid, force)
    }
}

/// Check if a process is a system process that should be filtered
pub fn is_system_process(name: &str) -> bool {
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
        "spotify", "chrome", "firefox", "slack", "discord", "code", // VS Code
        "teams", "zoom", "skype", "msedge", "brave",
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
    use crate::platform::MockPlatform;
    use crate::process::{ProcessInfo, ProcessStatus};

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
        assert_eq!(extract_command_description("", "postgres"), "postgres");
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

    // ===== MockPlatform Scanner Tests =====

    #[test]
    fn test_scanner_filters_system_processes() {
        let mock = MockPlatform::new()
            .with_port(3000, 1234)
            .with_process(
                1234,
                ProcessInfo {
                    pid: 1234,
                    name: "node.exe".to_string(),
                    command: "node server.js".to_string(),
                    status: ProcessStatus::Healthy,
                    memory_kb: 50000,
                    cpu_percent: 5.0,
                    start_time: None,
                    working_dir: Some("C:\\projects\\my-app".to_string()),
                },
            )
            .with_port(5000, 5678)
            .with_process(
                5678,
                ProcessInfo {
                    pid: 5678,
                    name: "svchost.exe".to_string(),
                    command: "C:\\Windows\\System32\\svchost.exe".to_string(),
                    status: ProcessStatus::Healthy,
                    memory_kb: 10000,
                    cpu_percent: 1.0,
                    start_time: None,
                    working_dir: None,
                },
            );

        let mut scanner = Scanner::new(Box::new(mock));
        let ports = scanner.scan(false).unwrap();

        // Should only include node.exe, not svchost.exe
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0].port, 3000);
        assert_eq!(ports[0].process_name, "server.js");
    }

    #[test]
    fn test_scanner_handles_process_exit() {
        let mock = MockPlatform::new()
            .with_port(3000, 1234)
            .with_error_on_pid(1234);

        let mut scanner = Scanner::new(Box::new(mock));
        let ports = scanner.scan(false).unwrap();

        // Should skip port with exited process
        assert_eq!(ports.len(), 0);
    }

    #[test]
    fn test_scanner_detects_zombie_processes() {
        let mock = MockPlatform::new().with_port(3000, 1234).with_process(
            1234,
            ProcessInfo {
                pid: 1234,
                name: "node.exe".to_string(),
                command: "node server.js".to_string(),
                status: ProcessStatus::Zombie,
                memory_kb: 0,
                cpu_percent: 0.0,
                start_time: None,
                working_dir: None,
            },
        );

        let mut scanner = Scanner::new(Box::new(mock));
        let ports = scanner.scan(false).unwrap();

        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0].status, ProcessStatus::Zombie);
    }

    #[test]
    fn test_scanner_detects_orphaned_processes() {
        let mock = MockPlatform::new().with_port(3000, 1234).with_process(
            1234,
            ProcessInfo {
                pid: 1234,
                name: "node.exe".to_string(),
                command: "node server.js".to_string(),
                status: ProcessStatus::Orphaned,
                memory_kb: 50000,
                cpu_percent: 5.0,
                start_time: None,
                working_dir: None,
            },
        );

        let mut scanner = Scanner::new(Box::new(mock));
        let ports = scanner.scan(false).unwrap();

        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0].status, ProcessStatus::Orphaned);
    }

    #[test]
    fn test_scanner_handles_empty_port_list() {
        let mock = MockPlatform::new();

        let mut scanner = Scanner::new(Box::new(mock));
        let ports = scanner.scan(false).unwrap();

        assert_eq!(ports.len(), 0);
    }

    #[test]
    fn test_scanner_handles_multiple_ports_same_pid() {
        let mock = MockPlatform::new()
            .with_port(3000, 1234)
            .with_port(3001, 1234)
            .with_port(3002, 1234)
            .with_process(
                1234,
                ProcessInfo {
                    pid: 1234,
                    name: "node.exe".to_string(),
                    command: "node server.js".to_string(),
                    status: ProcessStatus::Healthy,
                    memory_kb: 50000,
                    cpu_percent: 5.0,
                    start_time: None,
                    working_dir: Some("C:\\projects\\my-app".to_string()),
                },
            );

        let mut scanner = Scanner::new(Box::new(mock));
        let ports = scanner.scan(false).unwrap();

        assert_eq!(ports.len(), 3);
        assert_eq!(ports[0].port, 3000);
        assert_eq!(ports[1].port, 3001);
        assert_eq!(ports[2].port, 3002);
        assert_eq!(ports[0].pid, 1234);
        assert_eq!(ports[1].pid, 1234);
        assert_eq!(ports[2].pid, 1234);
    }

    #[test]
    fn test_scanner_show_all_includes_system_processes() {
        let mock = MockPlatform::new()
            .with_port(3000, 1234)
            .with_process(
                1234,
                ProcessInfo {
                    pid: 1234,
                    name: "node.exe".to_string(),
                    command: "node server.js".to_string(),
                    status: ProcessStatus::Healthy,
                    memory_kb: 50000,
                    cpu_percent: 5.0,
                    start_time: None,
                    working_dir: None,
                },
            )
            .with_port(5000, 5678)
            .with_process(
                5678,
                ProcessInfo {
                    pid: 5678,
                    name: "svchost.exe".to_string(),
                    command: "C:\\Windows\\System32\\svchost.exe".to_string(),
                    status: ProcessStatus::Healthy,
                    memory_kb: 10000,
                    cpu_percent: 1.0,
                    start_time: None,
                    working_dir: None,
                },
            );

        let mut scanner = Scanner::new(Box::new(mock));
        let ports = scanner.scan(true).unwrap(); // show_all = true

        // Should include both processes
        assert_eq!(ports.len(), 2);
    }

    #[test]
    fn test_scanner_handles_permission_denied() {
        let mock = MockPlatform::new()
            .with_port(3000, 1234)
            .with_error_on_pid(1234);

        let mut scanner = Scanner::new(Box::new(mock));

        // scan() should handle error gracefully and skip the port
        let ports = scanner.scan(false).unwrap();
        assert_eq!(ports.len(), 0);
    }

    // ========== Property-Based Tests ==========

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_scanner_handles_any_port(port in 1u16..=65535, pid in 1u32..=100000) {
            // Property: Scanner should handle any valid port number
            let mock = MockPlatform::new()
                .with_port(port, pid)
                .with_process(
                    pid,
                    ProcessInfo {
                        pid,
                        name: "test.exe".to_string(),
                        command: "test command".to_string(),
                        status: ProcessStatus::Healthy,
                        memory_kb: 50000,
                        cpu_percent: 5.0,
                        start_time: None,
                        working_dir: None,
                    },
                );

            let mut scanner = Scanner::new(Box::new(mock));
            let result = scanner.scan(false);

            // Should not panic and return valid result
            assert!(result.is_ok());
            let ports = result.unwrap();
            if !ports.is_empty() {
                assert_eq!(ports[0].port, port);
                assert_eq!(ports[0].pid, pid);
            }
        }

        #[test]
        fn prop_scanner_handles_any_pid(pid in 1u32..=100000) {
            // Property: Scanner should handle any PID
            let mock = MockPlatform::new()
                .with_port(3000, pid)
                .with_process(
                    pid,
                    ProcessInfo {
                        pid,
                        name: "test.exe".to_string(),
                        command: "test command".to_string(),
                        status: ProcessStatus::Healthy,
                        memory_kb: 50000,
                        cpu_percent: 5.0,
                        start_time: None,
                        working_dir: None,
                    },
                );

            let mut scanner = Scanner::new(Box::new(mock));
            let result = scanner.scan(false);

            assert!(result.is_ok());
        }

        #[test]
        fn prop_get_port_details_any_port(port in 1u16..=65535, pid in 1u32..=100000) {
            // Property: get_port_details should handle any port
            let mock = MockPlatform::new()
                .with_port(port, pid)
                .with_process(
                    pid,
                    ProcessInfo {
                        pid,
                        name: "test.exe".to_string(),
                        command: "test command".to_string(),
                        status: ProcessStatus::Healthy,
                        memory_kb: 50000,
                        cpu_percent: 5.0,
                        start_time: None,
                        working_dir: None,
                    },
                );

            let mut scanner = Scanner::new(Box::new(mock));
            let result = scanner.get_port_details(port);

            // Should either find the port or return PortNotFound error
            match result {
                Ok(port_info) => {
                    assert_eq!(port_info.port, port);
                    assert_eq!(port_info.pid, pid);
                }
                Err(_) => {
                    // Port not found is acceptable (filtered out)
                }
            }
        }

        #[test]
        fn prop_scanner_sorts_ports(
            port1 in 1u16..=30000,
            port2 in 30001u16..=60000,
            port3 in 60001u16..=65535
        ) {
            // Property: Scanner should always sort ports by port number
            let mock = MockPlatform::new()
                .with_port(port2, 2000) // Add middle port first
                .with_port(port3, 3000) // Add highest port second
                .with_port(port1, 1000) // Add lowest port last
                .with_process(
                    1000,
                    ProcessInfo {
                        pid: 1000,
                        name: "test1.exe".to_string(),
                        command: "test1".to_string(),
                        status: ProcessStatus::Healthy,
                        memory_kb: 50000,
                        cpu_percent: 5.0,
                        start_time: None,
                        working_dir: None,
                    },
                )
                .with_process(
                    2000,
                    ProcessInfo {
                        pid: 2000,
                        name: "test2.exe".to_string(),
                        command: "test2".to_string(),
                        status: ProcessStatus::Healthy,
                        memory_kb: 50000,
                        cpu_percent: 5.0,
                        start_time: None,
                        working_dir: None,
                    },
                )
                .with_process(
                    3000,
                    ProcessInfo {
                        pid: 3000,
                        name: "test3.exe".to_string(),
                        command: "test3".to_string(),
                        status: ProcessStatus::Healthy,
                        memory_kb: 50000,
                        cpu_percent: 5.0,
                        start_time: None,
                        working_dir: None,
                    },
                );

            let mut scanner = Scanner::new(Box::new(mock));
            let ports = scanner.scan(false).unwrap();

            // Should be sorted: port1 < port2 < port3
            if ports.len() == 3 {
                assert!(ports[0].port < ports[1].port);
                assert!(ports[1].port < ports[2].port);
                assert_eq!(ports[0].port, port1);
                assert_eq!(ports[1].port, port2);
                assert_eq!(ports[2].port, port3);
            }
        }

        #[test]
        fn prop_is_system_process_consistency(name in "[a-zA-Z0-9_.-]{1,50}\\.exe") {
            // Property: is_system_process should be deterministic
            let result1 = is_system_process(&name);
            let result2 = is_system_process(&name);

            assert_eq!(result1, result2);
        }

        #[test]
        fn prop_extract_command_description_no_panic(
            command in ".*",
            process_name in "[a-zA-Z0-9_.-]{1,20}"
        ) {
            // Property: extract_command_description should never panic
            let result = extract_command_description(&command, &process_name);

            // Should always return a non-empty string
            assert!(!result.is_empty());
        }
    }
}
