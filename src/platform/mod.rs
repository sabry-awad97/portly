use crate::error::Result;
use crate::process::{ProcessInfo, ProcessNode, RawPortInfo};

/// Platform abstraction trait for OS-specific operations.
///
/// Provides a common interface for platform-specific functionality
/// like port scanning, process management, and system information.
///
/// # Implementations
///
/// - `WindowsPlatform` - Windows-specific implementation using netstat2 and sysinfo
///
/// # Examples
///
/// ```no_run
/// use portly::platform::{Platform, get_platform};
///
/// let platform = get_platform();
/// let ports = platform.get_listening_ports()?;
/// # Ok::<(), portly::error::PortlyError>(())
/// ```
pub trait Platform {
    /// Get all listening ports with their PIDs.
    ///
    /// # Errors
    ///
    /// Returns error if port scanning fails
    fn get_listening_ports(&self) -> Result<Vec<RawPortInfo>>;

    /// Get detailed process information by PID.
    ///
    /// # Errors
    ///
    /// Returns error if process doesn't exist or info cannot be retrieved
    fn get_process_info(&self, pid: u32) -> Result<ProcessInfo>;

    /// Get process tree (ancestors) for a given PID.
    ///
    /// # Errors
    ///
    /// Returns error if process tree cannot be determined
    fn get_process_tree(&self, pid: u32) -> Result<Vec<ProcessNode>>;

    /// Kill a process by PID.
    ///
    /// # Arguments
    ///
    /// * `force` - If true, force kill; if false, graceful termination
    ///
    /// # Errors
    ///
    /// Returns error if process cannot be killed
    fn kill_process(&self, pid: u32, force: bool) -> Result<()>;

    /// Get all running processes (for ps command).
    ///
    /// # Errors
    ///
    /// Returns error if process list cannot be retrieved
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
        compile_error!(
            "Portly currently only supports Windows. macOS and Linux support coming soon."
        );
    }
}
