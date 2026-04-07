use crate::error::Result;
use crate::process::{ProcessInfo, ProcessNode, RawPortInfo};

/// Platform abstraction trait for OS-specific operations
pub trait Platform {
    /// Get all listening ports with their PIDs
    fn get_listening_ports(&self) -> Result<Vec<RawPortInfo>>;

    /// Get detailed process information by PID
    fn get_process_info(&self, pid: u32) -> Result<ProcessInfo>;

    /// Get process tree (ancestors) for a given PID
    fn get_process_tree(&self, pid: u32) -> Result<Vec<ProcessNode>>;

    /// Kill a process by PID
    fn kill_process(&self, pid: u32, force: bool) -> Result<()>;

    /// Get all running processes (for ps command)
    fn get_all_processes(&self) -> Result<Vec<ProcessInfo>>;
}

// Platform-specific implementations
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub use windows::WindowsPlatform;

/// Get the platform implementation for the current OS
pub fn get_platform() -> Box<dyn Platform> {
    #[cfg(target_os = "windows")]
    {
        Box::new(WindowsPlatform::new())
    }

    #[cfg(not(target_os = "windows"))]
    {
        compile_error!("Portly currently only supports Windows. macOS and Linux support coming soon.");
    }
}
