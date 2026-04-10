use crate::error::{PortlyError, Result};
use crate::platform::Platform;
use crate::process::{ProcessInfo, ProcessNode, ProcessStatus, RawPortInfo};
use netstat2::{AddressFamilyFlags, ProtocolFlags, ProtocolSocketInfo, TcpState, get_sockets_info};
use sysinfo::{Pid, System};

/// Native platform implementation using cross-platform crates
pub struct NativePlatform {
    system: System,
}

impl NativePlatform {
    pub fn new() -> Self {
        let mut system = System::new();
        system.refresh_all();
        Self { system }
    }
}

impl Platform for NativePlatform {
    fn get_listening_ports(&self) -> Result<Vec<RawPortInfo>> {
        let af_flags = AddressFamilyFlags::IPV4 | AddressFamilyFlags::IPV6;
        let proto_flags = ProtocolFlags::TCP | ProtocolFlags::UDP;

        let sockets = get_sockets_info(af_flags, proto_flags)
            .map_err(|e| PortlyError::PlatformError(format!("Failed to get socket info: {}", e)))?;

        let mut ports = Vec::new();

        for socket in sockets {
            // Only include listening TCP ports
            let is_listening = matches!(
                socket.protocol_socket_info,
                ProtocolSocketInfo::Tcp(ref tcp) if tcp.state == TcpState::Listen
            );

            if !is_listening {
                continue;
            }

            let port = match socket.protocol_socket_info {
                ProtocolSocketInfo::Tcp(tcp) => tcp.local_port,
                ProtocolSocketInfo::Udp(udp) => udp.local_port,
            };

            // Get first PID (most sockets have one PID)
            if let Some(&pid) = socket.associated_pids.first() {
                ports.push(RawPortInfo { port, pid });
            }
        }

        Ok(ports)
    }

    fn get_process_info(&self, pid: u32) -> Result<ProcessInfo> {
        let process =
            self.system
                .process(Pid::from_u32(pid))
                .ok_or(PortlyError::ProcessNotFound {
                    pid,
                    suggestion: Some(
                        "• The process may have exited\n\
                     • Run 'portly list' to see current processes\n\
                     • Check if you have permission to access this process"
                            .to_string(),
                    ),
                })?;

        let name = process.name().to_string_lossy().to_string();
        let command = process
            .cmd()
            .iter()
            .map(|s| s.to_string_lossy().to_string())
            .collect::<Vec<_>>()
            .join(" ");
        let memory_kb = process.memory() / 1024; // Convert bytes to KB
        let cpu_percent = process.cpu_usage();
        let start_time =
            Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(process.start_time()));
        let working_dir = process.cwd().map(|p| p.to_string_lossy().to_string());

        // Determine process status
        let status = if process.memory() == 0 {
            ProcessStatus::Zombie
        } else if process.parent().is_none() && pid > 1000 {
            ProcessStatus::Orphaned
        } else {
            ProcessStatus::Healthy
        };

        Ok(ProcessInfo {
            pid,
            name,
            command,
            status,
            memory_kb,
            cpu_percent,
            start_time,
            working_dir,
        })
    }

    fn get_process_tree(&self, pid: u32) -> Result<Vec<ProcessNode>> {
        let mut tree = Vec::new();
        let mut current_pid = pid;
        let mut depth = 0;

        // Traverse up the process tree (max 8 levels)
        while depth < 8 {
            if let Some(process) = self.system.process(Pid::from_u32(current_pid)) {
                let ppid = process.parent().map(|p| p.as_u32()).unwrap_or(0);
                let name = process.name().to_string_lossy().to_string();

                tree.push(ProcessNode {
                    pid: current_pid,
                    ppid,
                    name,
                });

                if ppid == 0 || ppid == current_pid {
                    break;
                }

                current_pid = ppid;
                depth += 1;
            } else {
                break;
            }
        }

        Ok(tree)
    }

    fn kill_process(&self, pid: u32, force: bool) -> Result<()> {
        // Get process reference
        let process =
            self.system
                .process(Pid::from_u32(pid))
                .ok_or(PortlyError::ProcessNotFound {
                    pid,
                    suggestion: Some(
                        "• The process may have exited\n\
                     • Run 'portly list' to see current processes\n\
                     • Check if you have permission to access this process"
                            .to_string(),
                    ),
                })?;

        // Use sysinfo's cross-platform kill method
        // force=true: Send SIGKILL (immediate termination)
        // force=false: Send SIGTERM (graceful termination)
        let signal = if force {
            sysinfo::Signal::Kill
        } else {
            sysinfo::Signal::Term
        };

        match process.kill_with(signal) {
            Some(true) => Ok(()),
            Some(false) => Err(PortlyError::PlatformError(format!(
                "Failed to kill process {}: Permission denied or process protected",
                pid
            ))),
            None => Err(PortlyError::PlatformError(format!(
                "Failed to kill process {}: Signal not supported on this platform",
                pid
            ))),
        }
    }

    fn get_all_processes(&self) -> Result<Vec<ProcessInfo>> {
        let mut processes = Vec::new();

        for (pid, process) in self.system.processes() {
            let name = process.name().to_string_lossy().to_string();
            let command = process
                .cmd()
                .iter()
                .map(|s| s.to_string_lossy().to_string())
                .collect::<Vec<_>>()
                .join(" ");
            let memory_kb = process.memory() / 1024;
            let cpu_percent = process.cpu_usage();
            let start_time =
                Some(std::time::UNIX_EPOCH + std::time::Duration::from_secs(process.start_time()));
            let working_dir = process.cwd().map(|p| p.to_string_lossy().to_string());

            let status = if process.memory() == 0 {
                ProcessStatus::Zombie
            } else if process.parent().is_none() && pid.as_u32() > 1000 {
                ProcessStatus::Orphaned
            } else {
                ProcessStatus::Healthy
            };

            processes.push(ProcessInfo {
                pid: pid.as_u32(),
                name,
                command,
                status,
                memory_kb,
                cpu_percent,
                start_time,
                working_dir,
            });
        }

        Ok(processes)
    }
}

impl Default for NativePlatform {
    fn default() -> Self {
        Self::new()
    }
}
