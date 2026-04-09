#[cfg(test)]
use crate::error::{PortlyError, Result};
#[cfg(test)]
use crate::platform::Platform;
#[cfg(test)]
use crate::process::{ProcessInfo, ProcessNode, RawPortInfo};
#[cfg(test)]
use std::collections::HashMap;
#[cfg(test)]
use std::sync::{Arc, Mutex};

/// Mock platform implementation for testing.
///
/// Provides in-memory data structures to simulate platform operations
/// without requiring real OS interactions. Useful for test-driven development.
///
/// # Examples
///
/// ```
/// use portly::platform::{Platform, MockPlatform};
/// use portly::process::RawPortInfo;
///
/// let mut mock = MockPlatform::new();
/// mock.ports.push(RawPortInfo { port: 3000, pid: 1234 });
///
/// let ports = mock.get_listening_ports().unwrap();
/// assert_eq!(ports.len(), 1);
/// assert_eq!(ports[0].port, 3000);
/// ```
#[cfg(test)]
#[derive(Debug, Default)]
pub struct MockPlatform {
    /// In-memory storage for listening ports
    pub ports: Vec<RawPortInfo>,

    /// In-memory storage for process information (keyed by PID)
    pub processes: HashMap<u32, ProcessInfo>,

    /// In-memory storage for process trees (keyed by PID)
    pub process_trees: HashMap<u32, Vec<ProcessNode>>,

    /// Records of kill_process calls: (pid, force)
    /// Uses Arc<Mutex<>> for interior mutability since Platform trait requires &self
    pub kill_calls: Arc<Mutex<Vec<(u32, bool)>>>,

    /// Port to simulate errors on (for testing error handling)
    error_on_port: Option<u16>,

    /// PID to simulate errors on (for testing error handling)
    error_on_pid: Option<u32>,
}

#[cfg(test)]
impl MockPlatform {
    /// Create a new empty MockPlatform
    pub fn new() -> Self {
        Self {
            ports: Vec::new(),
            processes: HashMap::new(),
            process_trees: HashMap::new(),
            kill_calls: Arc::new(Mutex::new(Vec::new())),
            error_on_port: None,
            error_on_pid: None,
        }
    }

    /// Builder method: Add a port mapping
    ///
    /// # Examples
    ///
    /// ```
    /// use portly::platform::MockPlatform;
    ///
    /// let mock = MockPlatform::new()
    ///     .with_port(3000, 1234)
    ///     .with_port(8080, 5678);
    /// ```
    pub fn with_port(mut self, port: u16, pid: u32) -> Self {
        self.ports.push(RawPortInfo { port, pid });
        self
    }

    /// Builder method: Add process information
    ///
    /// # Examples
    ///
    /// ```
    /// use portly::platform::MockPlatform;
    /// use portly::process::{ProcessInfo, ProcessStatus};
    ///
    /// let process = ProcessInfo {
    ///     pid: 1234,
    ///     name: "node.exe".to_string(),
    ///     command: "node server.js".to_string(),
    ///     status: ProcessStatus::Healthy,
    ///     memory_kb: 50000,
    ///     cpu_percent: 2.5,
    ///     start_time: None,
    ///     working_dir: None,
    /// };
    ///
    /// let mock = MockPlatform::new().with_process(1234, process);
    /// ```
    pub fn with_process(mut self, pid: u32, info: ProcessInfo) -> Self {
        self.processes.insert(pid, info);
        self
    }

    /// Builder method: Add process tree
    ///
    /// # Examples
    ///
    /// ```
    /// use portly::platform::MockPlatform;
    /// use portly::process::ProcessNode;
    ///
    /// let tree = vec![
    ///     ProcessNode { pid: 1234, ppid: 5678, name: "node.exe".to_string() },
    ///     ProcessNode { pid: 5678, ppid: 0, name: "cmd.exe".to_string() },
    /// ];
    ///
    /// let mock = MockPlatform::new().with_process_tree(1234, tree);
    /// ```
    pub fn with_process_tree(mut self, pid: u32, tree: Vec<ProcessNode>) -> Self {
        self.process_trees.insert(pid, tree);
        self
    }

    /// Builder method: Configure error simulation for a specific port
    ///
    /// When configured, any operation involving this port will return a simulated error.
    /// Useful for testing error handling paths.
    ///
    /// # Examples
    ///
    /// ```
    /// use portly::platform::{Platform, MockPlatform};
    ///
    /// let mock = MockPlatform::new()
    ///     .with_port(3000, 1234)
    ///     .with_error_on_port(3000);
    ///
    /// let result = mock.get_listening_ports();
    /// assert!(result.is_err());
    /// ```
    pub fn with_error_on_port(mut self, port: u16) -> Self {
        self.error_on_port = Some(port);
        self
    }

    /// Builder method: Configure error simulation for a specific PID
    ///
    /// When configured, any operation involving this PID will return a simulated error.
    /// Useful for testing error handling paths like permission denied or process exit scenarios.
    ///
    /// # Examples
    ///
    /// ```
    /// use portly::platform::{Platform, MockPlatform};
    /// use portly::process::{ProcessInfo, ProcessStatus};
    ///
    /// let process = ProcessInfo {
    ///     pid: 1234,
    ///     name: "node.exe".to_string(),
    ///     command: "node server.js".to_string(),
    ///     status: ProcessStatus::Healthy,
    ///     memory_kb: 50000,
    ///     cpu_percent: 2.5,
    ///     start_time: None,
    ///     working_dir: None,
    /// };
    ///
    /// let mock = MockPlatform::new()
    ///     .with_process(1234, process)
    ///     .with_error_on_pid(1234);
    ///
    /// let result = mock.get_process_info(1234);
    /// assert!(result.is_err());
    /// ```
    pub fn with_error_on_pid(mut self, pid: u32) -> Self {
        self.error_on_pid = Some(pid);
        self
    }

    /// Assertion helper: Verify a process was killed
    ///
    /// Panics if the process was not killed.
    ///
    /// # Examples
    ///
    /// ```should_panic
    /// use portly::platform::{Platform, MockPlatform};
    /// use portly::process::{ProcessInfo, ProcessStatus};
    ///
    /// let process = ProcessInfo {
    ///     pid: 1234,
    ///     name: "node.exe".to_string(),
    ///     command: "node server.js".to_string(),
    ///     status: ProcessStatus::Healthy,
    ///     memory_kb: 50000,
    ///     cpu_percent: 2.5,
    ///     start_time: None,
    ///     working_dir: None,
    /// };
    ///
    /// let mock = MockPlatform::new().with_process(1234, process);
    /// let _ = mock.kill_process(1234, false);
    /// mock.assert_killed(1234);
    /// ```
    pub fn assert_killed(&self, pid: u32) {
        let calls = self.kill_calls.lock().unwrap();
        if !calls.iter().any(|(p, _)| *p == pid) {
            panic!(
                "Expected process {} to be killed, but it was not. Kill calls: {:?}",
                pid, *calls
            );
        }
    }

    /// Assertion helper: Verify a process was NOT killed
    ///
    /// Panics if the process was killed.
    ///
    /// # Examples
    ///
    /// ```
    /// use portly::platform::MockPlatform;
    ///
    /// let mock = MockPlatform::new();
    /// mock.assert_not_killed(1234);
    /// ```
    pub fn assert_not_killed(&self, pid: u32) {
        let calls = self.kill_calls.lock().unwrap();
        if calls.iter().any(|(p, _)| *p == pid) {
            panic!(
                "Expected process {} to NOT be killed, but it was. Kill calls: {:?}",
                pid, *calls
            );
        }
    }

    /// Assertion helper: Verify the total number of kill calls
    ///
    /// Panics if the count doesn't match.
    ///
    /// # Examples
    ///
    /// ```
    /// use portly::platform::{Platform, MockPlatform};
    /// use portly::process::{ProcessInfo, ProcessStatus};
    ///
    /// let process1 = ProcessInfo {
    ///     pid: 1234,
    ///     name: "node.exe".to_string(),
    ///     command: "node server.js".to_string(),
    ///     status: ProcessStatus::Healthy,
    ///     memory_kb: 50000,
    ///     cpu_percent: 2.5,
    ///     start_time: None,
    ///     working_dir: None,
    /// };
    ///
    /// let process2 = ProcessInfo {
    ///     pid: 5678,
    ///     name: "python.exe".to_string(),
    ///     command: "python app.py".to_string(),
    ///     status: ProcessStatus::Healthy,
    ///     memory_kb: 80000,
    ///     cpu_percent: 5.0,
    ///     start_time: None,
    ///     working_dir: None,
    /// };
    ///
    /// let mock = MockPlatform::new()
    ///     .with_process(1234, process1)
    ///     .with_process(5678, process2);
    ///
    /// let _ = mock.kill_process(1234, false);
    /// let _ = mock.kill_process(5678, true);
    /// mock.assert_kill_count(2);
    /// ```
    pub fn assert_kill_count(&self, expected: usize) {
        let calls = self.kill_calls.lock().unwrap();
        let actual = calls.len();
        if actual != expected {
            panic!(
                "Expected {} kill calls, but got {}. Kill calls: {:?}",
                expected, actual, *calls
            );
        }
    }

    /// Get all kill calls for custom assertions
    ///
    /// Returns a vector of (pid, force) tuples representing all kill_process calls.
    ///
    /// # Examples
    ///
    /// ```
    /// use portly::platform::{Platform, MockPlatform};
    /// use portly::process::{ProcessInfo, ProcessStatus};
    ///
    /// let process = ProcessInfo {
    ///     pid: 1234,
    ///     name: "node.exe".to_string(),
    ///     command: "node server.js".to_string(),
    ///     status: ProcessStatus::Healthy,
    ///     memory_kb: 50000,
    ///     cpu_percent: 2.5,
    ///     start_time: None,
    ///     working_dir: None,
    /// };
    ///
    /// let mock = MockPlatform::new().with_process(1234, process);
    /// let _ = mock.kill_process(1234, true);
    ///
    /// let calls = mock.get_kill_calls();
    /// assert_eq!(calls.len(), 1);
    /// assert_eq!(calls[0], (1234, true));
    /// ```
    pub fn get_kill_calls(&self) -> Vec<(u32, bool)> {
        self.kill_calls.lock().unwrap().clone()
    }

    /// Builder method: Create a chain of processes (parent -> child -> grandchild -> ...)
    ///
    /// Creates a linear process tree with the specified depth.
    /// Each process in the chain has the next one as its parent.
    ///
    /// # Examples
    ///
    /// ```
    /// use portly::platform::MockPlatform;
    ///
    /// // Create a chain: 1000 -> 1001 -> 1002 -> 1003
    /// let mock = MockPlatform::new().with_process_chain(1000, 4);
    /// ```
    pub fn with_process_chain(mut self, root_pid: u32, depth: usize) -> Self {
        if depth == 0 {
            return self;
        }

        let mut tree = Vec::new();
        let mut current_pid = root_pid;

        for i in 0..depth {
            let ppid = if i == 0 { 0 } else { current_pid - 1 };
            tree.push(ProcessNode {
                pid: current_pid,
                ppid,
                name: format!("process_{}.exe", current_pid),
            });
            current_pid += 1;
        }

        self.process_trees.insert(root_pid, tree);
        self
    }

    /// Builder method: Create a process tree with a cycle (for testing cycle detection)
    ///
    /// Creates a process tree where a child process has the root as its parent,
    /// creating a cycle. This is useful for testing cycle detection logic.
    ///
    /// # Examples
    ///
    /// ```
    /// use portly::platform::MockPlatform;
    ///
    /// // Create a cycle: 1000 -> 1001 -> 1000 (cycle!)
    /// let mock = MockPlatform::new().with_process_tree_cycle(1000);
    /// ```
    pub fn with_process_tree_cycle(mut self, root_pid: u32) -> Self {
        let tree = vec![
            ProcessNode {
                pid: root_pid,
                ppid: root_pid + 1, // Parent is the child (cycle!)
                name: format!("process_{}.exe", root_pid),
            },
            ProcessNode {
                pid: root_pid + 1,
                ppid: root_pid, // Child's parent is root
                name: format!("process_{}.exe", root_pid + 1),
            },
        ];

        self.process_trees.insert(root_pid, tree);
        self
    }
}

#[cfg(test)]
impl Platform for MockPlatform {
    fn get_listening_ports(&self) -> Result<Vec<RawPortInfo>> {
        // Check if any port in the list matches the error_on_port configuration
        if let Some(error_port) = self.error_on_port {
            if self.ports.iter().any(|p| p.port == error_port) {
                return Err(PortlyError::PlatformError(format!(
                    "Simulated error for port {}",
                    error_port
                )));
            }
        }
        Ok(self.ports.clone())
    }

    fn get_process_info(&self, pid: u32) -> Result<ProcessInfo> {
        // Check for simulated error first
        if self.error_on_pid == Some(pid) {
            return Err(PortlyError::PlatformError(format!(
                "Simulated error for PID {}",
                pid
            )));
        }

        self.processes
            .get(&pid)
            .cloned()
            .ok_or(PortlyError::ProcessNotFound {
                pid,
                suggestion: Some(
                    "• The process may have exited\n\
                     • Run 'portly list' to see current processes\n\
                     • Check if you have permission to access this process".to_string()
                ),
            })
    }

    fn get_process_tree(&self, pid: u32) -> Result<Vec<ProcessNode>> {
        // Check for simulated error first
        if self.error_on_pid == Some(pid) {
            return Err(PortlyError::PlatformError(format!(
                "Simulated error for PID {}",
                pid
            )));
        }

        self.process_trees
            .get(&pid)
            .cloned()
            .ok_or(PortlyError::ProcessNotFound {
                pid,
                suggestion: Some(
                    "• The process may have exited\n\
                     • Run 'portly list' to see current processes\n\
                     • Check if you have permission to access this process".to_string()
                ),
            })
    }

    fn kill_process(&self, pid: u32, force: bool) -> Result<()> {
        // Check for simulated error first (before recording kill call)
        if self.error_on_pid == Some(pid) {
            return Err(PortlyError::PlatformError(format!(
                "Simulated error for PID {}",
                pid
            )));
        }

        // Verify process exists
        if !self.processes.contains_key(&pid) {
            return Err(PortlyError::ProcessNotFound {
                pid,
                suggestion: Some(
                    "• The process may have exited\n\
                     • Run 'portly list' to see current processes\n\
                     • Check if you have permission to access this process".to_string()
                ),
            });
        }

        // Record the kill call using interior mutability
        self.kill_calls.lock().unwrap().push((pid, force));
        Ok(())
    }

    fn get_all_processes(&self) -> Result<Vec<ProcessInfo>> {
        Ok(self.processes.values().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::ProcessStatus;

    #[test]
    fn test_mock_platform_new() {
        let mock = MockPlatform::new();
        assert!(mock.ports.is_empty());
        assert!(mock.processes.is_empty());
        assert!(mock.process_trees.is_empty());
        assert!(mock.kill_calls.lock().unwrap().is_empty());
    }

    #[test]
    fn test_get_listening_ports() {
        let mut mock = MockPlatform::new();
        mock.ports.push(RawPortInfo {
            port: 3000,
            pid: 1234,
        });
        mock.ports.push(RawPortInfo {
            port: 8080,
            pid: 5678,
        });

        let ports = mock.get_listening_ports().unwrap();
        assert_eq!(ports.len(), 2);
        assert_eq!(ports[0].port, 3000);
        assert_eq!(ports[0].pid, 1234);
        assert_eq!(ports[1].port, 8080);
        assert_eq!(ports[1].pid, 5678);
    }

    #[test]
    fn test_get_process_info_success() {
        let mut mock = MockPlatform::new();
        let process = ProcessInfo {
            pid: 1234,
            name: "node.exe".to_string(),
            command: "node server.js".to_string(),
            status: ProcessStatus::Healthy,
            memory_kb: 50000,
            cpu_percent: 2.5,
            start_time: None,
            working_dir: Some("C:\\projects\\app".to_string()),
        };
        mock.processes.insert(1234, process.clone());

        let result = mock.get_process_info(1234).unwrap();
        assert_eq!(result, process);
    }

    #[test]
    fn test_get_process_info_not_found() {
        let mock = MockPlatform::new();
        let result = mock.get_process_info(9999);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PortlyError::ProcessNotFound { pid: 9999, .. }
        ));
    }

    #[test]
    fn test_get_process_tree_success() {
        let mut mock = MockPlatform::new();
        let tree = vec![
            ProcessNode {
                pid: 1234,
                ppid: 5678,
                name: "node.exe".to_string(),
            },
            ProcessNode {
                pid: 5678,
                ppid: 0,
                name: "cmd.exe".to_string(),
            },
        ];
        mock.process_trees.insert(1234, tree.clone());

        let result = mock.get_process_tree(1234).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].pid, 1234);
        assert_eq!(result[1].pid, 5678);
    }

    #[test]
    fn test_get_process_tree_not_found() {
        let mock = MockPlatform::new();
        let result = mock.get_process_tree(9999);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PortlyError::ProcessNotFound { pid: 9999, .. }
        ));
    }

    #[test]
    fn test_kill_process_success() {
        let mut mock = MockPlatform::new();
        let process = ProcessInfo {
            pid: 1234,
            name: "node.exe".to_string(),
            command: "node server.js".to_string(),
            status: ProcessStatus::Healthy,
            memory_kb: 50000,
            cpu_percent: 2.5,
            start_time: None,
            working_dir: None,
        };
        mock.processes.insert(1234, process);

        let result = mock.kill_process(1234, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_kill_process_not_found() {
        let mock = MockPlatform::new();
        let result = mock.kill_process(9999, false);
        assert!(result.is_err());
        assert!(matches!(
            result.unwrap_err(),
            PortlyError::ProcessNotFound { pid: 9999, .. }
        ));
    }

    #[test]
    fn test_get_all_processes() {
        let mut mock = MockPlatform::new();

        let process1 = ProcessInfo {
            pid: 1234,
            name: "node.exe".to_string(),
            command: "node server.js".to_string(),
            status: ProcessStatus::Healthy,
            memory_kb: 50000,
            cpu_percent: 2.5,
            start_time: None,
            working_dir: None,
        };

        let process2 = ProcessInfo {
            pid: 5678,
            name: "python.exe".to_string(),
            command: "python app.py".to_string(),
            status: ProcessStatus::Healthy,
            memory_kb: 80000,
            cpu_percent: 5.0,
            start_time: None,
            working_dir: None,
        };

        mock.processes.insert(1234, process1);
        mock.processes.insert(5678, process2);

        let all_processes = mock.get_all_processes().unwrap();
        assert_eq!(all_processes.len(), 2);

        // Check that both processes are present (order may vary due to HashMap)
        let pids: Vec<u32> = all_processes.iter().map(|p| p.pid).collect();
        assert!(pids.contains(&1234));
        assert!(pids.contains(&5678));
    }

    #[test]
    fn test_get_all_processes_empty() {
        let mock = MockPlatform::new();
        let all_processes = mock.get_all_processes().unwrap();
        assert!(all_processes.is_empty());
    }

    // ===== Builder Pattern Tests =====

    #[test]
    fn test_builder_with_port() {
        let mock = MockPlatform::new()
            .with_port(3000, 1234)
            .with_port(8080, 5678);

        let ports = mock.get_listening_ports().unwrap();
        assert_eq!(ports.len(), 2);
        assert_eq!(ports[0].port, 3000);
        assert_eq!(ports[0].pid, 1234);
        assert_eq!(ports[1].port, 8080);
        assert_eq!(ports[1].pid, 5678);
    }

    #[test]
    fn test_builder_with_process() {
        let process1 = ProcessInfo {
            pid: 1234,
            name: "node.exe".to_string(),
            command: "node server.js".to_string(),
            status: ProcessStatus::Healthy,
            memory_kb: 50000,
            cpu_percent: 2.5,
            start_time: None,
            working_dir: None,
        };

        let process2 = ProcessInfo {
            pid: 5678,
            name: "python.exe".to_string(),
            command: "python app.py".to_string(),
            status: ProcessStatus::Healthy,
            memory_kb: 80000,
            cpu_percent: 5.0,
            start_time: None,
            working_dir: None,
        };

        let mock = MockPlatform::new()
            .with_process(1234, process1.clone())
            .with_process(5678, process2.clone());

        let result1 = mock.get_process_info(1234).unwrap();
        assert_eq!(result1, process1);

        let result2 = mock.get_process_info(5678).unwrap();
        assert_eq!(result2, process2);
    }

    #[test]
    fn test_builder_with_process_tree() {
        let tree = vec![
            ProcessNode {
                pid: 1234,
                ppid: 5678,
                name: "node.exe".to_string(),
            },
            ProcessNode {
                pid: 5678,
                ppid: 0,
                name: "cmd.exe".to_string(),
            },
        ];

        let mock = MockPlatform::new().with_process_tree(1234, tree.clone());

        let result = mock.get_process_tree(1234).unwrap();
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].pid, 1234);
        assert_eq!(result[1].pid, 5678);
    }

    #[test]
    fn test_builder_chaining_all_methods() {
        let process = ProcessInfo {
            pid: 1234,
            name: "node.exe".to_string(),
            command: "node server.js".to_string(),
            status: ProcessStatus::Healthy,
            memory_kb: 50000,
            cpu_percent: 2.5,
            start_time: None,
            working_dir: None,
        };

        let tree = vec![ProcessNode {
            pid: 1234,
            ppid: 0,
            name: "node.exe".to_string(),
        }];

        let mock = MockPlatform::new()
            .with_port(3000, 1234)
            .with_process(1234, process.clone())
            .with_process_tree(1234, tree.clone());

        // Verify all data was set correctly
        let ports = mock.get_listening_ports().unwrap();
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0].port, 3000);

        let proc = mock.get_process_info(1234).unwrap();
        assert_eq!(proc, process);

        let proc_tree = mock.get_process_tree(1234).unwrap();
        assert_eq!(proc_tree.len(), 1);
    }

    // ===== Assertion Helper Tests =====

    #[test]
    fn test_assert_killed_success() {
        let process = ProcessInfo {
            pid: 1234,
            name: "node.exe".to_string(),
            command: "node server.js".to_string(),
            status: ProcessStatus::Healthy,
            memory_kb: 50000,
            cpu_percent: 2.5,
            start_time: None,
            working_dir: None,
        };

        let mock = MockPlatform::new().with_process(1234, process);
        mock.kill_process(1234, false).unwrap();
        mock.assert_killed(1234); // Should not panic
    }

    #[test]
    #[should_panic(expected = "Expected process 1234 to be killed")]
    fn test_assert_killed_failure() {
        let mock = MockPlatform::new();
        mock.assert_killed(1234); // Should panic
    }

    #[test]
    fn test_assert_not_killed_success() {
        let mock = MockPlatform::new();
        mock.assert_not_killed(1234); // Should not panic
    }

    #[test]
    #[should_panic(expected = "Expected process 1234 to NOT be killed")]
    fn test_assert_not_killed_failure() {
        let process = ProcessInfo {
            pid: 1234,
            name: "node.exe".to_string(),
            command: "node server.js".to_string(),
            status: ProcessStatus::Healthy,
            memory_kb: 50000,
            cpu_percent: 2.5,
            start_time: None,
            working_dir: None,
        };

        let mock = MockPlatform::new().with_process(1234, process);
        mock.kill_process(1234, false).unwrap();
        mock.assert_not_killed(1234); // Should panic
    }

    #[test]
    fn test_assert_kill_count_success() {
        let process1 = ProcessInfo {
            pid: 1234,
            name: "node.exe".to_string(),
            command: "node server.js".to_string(),
            status: ProcessStatus::Healthy,
            memory_kb: 50000,
            cpu_percent: 2.5,
            start_time: None,
            working_dir: None,
        };

        let process2 = ProcessInfo {
            pid: 5678,
            name: "python.exe".to_string(),
            command: "python app.py".to_string(),
            status: ProcessStatus::Healthy,
            memory_kb: 80000,
            cpu_percent: 5.0,
            start_time: None,
            working_dir: None,
        };

        let mock = MockPlatform::new()
            .with_process(1234, process1)
            .with_process(5678, process2);

        mock.kill_process(1234, false).unwrap();
        mock.kill_process(5678, true).unwrap();
        mock.assert_kill_count(2); // Should not panic
    }

    #[test]
    #[should_panic(expected = "Expected 3 kill calls, but got 2")]
    fn test_assert_kill_count_failure() {
        let process1 = ProcessInfo {
            pid: 1234,
            name: "node.exe".to_string(),
            command: "node server.js".to_string(),
            status: ProcessStatus::Healthy,
            memory_kb: 50000,
            cpu_percent: 2.5,
            start_time: None,
            working_dir: None,
        };

        let process2 = ProcessInfo {
            pid: 5678,
            name: "python.exe".to_string(),
            command: "python app.py".to_string(),
            status: ProcessStatus::Healthy,
            memory_kb: 80000,
            cpu_percent: 5.0,
            start_time: None,
            working_dir: None,
        };

        let mock = MockPlatform::new()
            .with_process(1234, process1)
            .with_process(5678, process2);

        mock.kill_process(1234, false).unwrap();
        mock.kill_process(5678, true).unwrap();
        mock.assert_kill_count(3); // Should panic
    }

    #[test]
    fn test_get_kill_calls() {
        let process1 = ProcessInfo {
            pid: 1234,
            name: "node.exe".to_string(),
            command: "node server.js".to_string(),
            status: ProcessStatus::Healthy,
            memory_kb: 50000,
            cpu_percent: 2.5,
            start_time: None,
            working_dir: None,
        };

        let process2 = ProcessInfo {
            pid: 5678,
            name: "python.exe".to_string(),
            command: "python app.py".to_string(),
            status: ProcessStatus::Healthy,
            memory_kb: 80000,
            cpu_percent: 5.0,
            start_time: None,
            working_dir: None,
        };

        let mock = MockPlatform::new()
            .with_process(1234, process1)
            .with_process(5678, process2);

        mock.kill_process(1234, false).unwrap();
        mock.kill_process(5678, true).unwrap();

        let calls = mock.get_kill_calls();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0], (1234, false));
        assert_eq!(calls[1], (5678, true));
    }

    #[test]
    fn test_get_kill_calls_empty() {
        let mock = MockPlatform::new();
        let calls = mock.get_kill_calls();
        assert!(calls.is_empty());
    }

    #[test]
    fn test_kill_process_records_force_flag() {
        let process = ProcessInfo {
            pid: 1234,
            name: "node.exe".to_string(),
            command: "node server.js".to_string(),
            status: ProcessStatus::Healthy,
            memory_kb: 50000,
            cpu_percent: 2.5,
            start_time: None,
            working_dir: None,
        };

        let mock = MockPlatform::new().with_process(1234, process);

        // Kill with force=false
        mock.kill_process(1234, false).unwrap();
        let calls = mock.get_kill_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].1, false);

        // Kill with force=true
        mock.kill_process(1234, true).unwrap();
        let calls = mock.get_kill_calls();
        assert_eq!(calls.len(), 2);
        assert_eq!(calls[1].1, true);
    }

    // ===== Error Simulation Tests =====

    #[test]
    fn test_error_simulation_on_port() {
        let mock = MockPlatform::new()
            .with_port(3000, 1234)
            .with_error_on_port(3000);

        let result = mock.get_listening_ports();
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Simulated error for port 3000"));
    }

    #[test]
    fn test_error_simulation_on_pid_get_process_info() {
        let process = ProcessInfo {
            pid: 1234,
            name: "node.exe".to_string(),
            command: "node server.js".to_string(),
            status: ProcessStatus::Healthy,
            memory_kb: 50000,
            cpu_percent: 2.5,
            start_time: None,
            working_dir: None,
        };

        let mock = MockPlatform::new()
            .with_process(1234, process)
            .with_error_on_pid(1234);

        let result = mock.get_process_info(1234);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Simulated error for PID 1234"));
    }

    #[test]
    fn test_error_simulation_on_pid_get_process_tree() {
        let tree = vec![ProcessNode {
            pid: 1234,
            ppid: 0,
            name: "node.exe".to_string(),
        }];

        let mock = MockPlatform::new()
            .with_process_tree(1234, tree)
            .with_error_on_pid(1234);

        let result = mock.get_process_tree(1234);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Simulated error for PID 1234"));
    }

    #[test]
    fn test_error_simulation_on_pid_kill_process() {
        let process = ProcessInfo {
            pid: 1234,
            name: "node.exe".to_string(),
            command: "node server.js".to_string(),
            status: ProcessStatus::Healthy,
            memory_kb: 50000,
            cpu_percent: 2.5,
            start_time: None,
            working_dir: None,
        };

        let mock = MockPlatform::new()
            .with_process(1234, process)
            .with_error_on_pid(1234);

        let result = mock.kill_process(1234, false);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("Simulated error for PID 1234"));

        // Verify kill was NOT recorded (error happened before recording)
        let calls = mock.get_kill_calls();
        assert!(calls.is_empty());
    }

    #[test]
    fn test_process_exit_scenario() {
        // Simulate process exiting between port scan and process info lookup
        let mock = MockPlatform::new()
            .with_port(3000, 1234)
            .with_error_on_pid(1234);

        // Port scan succeeds
        let ports = mock.get_listening_ports().unwrap();
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0].port, 3000);
        assert_eq!(ports[0].pid, 1234);

        // Process info fails (process exited)
        let result = mock.get_process_info(1234);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Simulated error"));
    }

    #[test]
    fn test_permission_denied_scenario() {
        // Simulate permission denied when trying to kill process
        let process = ProcessInfo {
            pid: 1234,
            name: "system.exe".to_string(),
            command: "system process".to_string(),
            status: ProcessStatus::Healthy,
            memory_kb: 10000,
            cpu_percent: 1.0,
            start_time: None,
            working_dir: None,
        };

        let mock = MockPlatform::new()
            .with_process(1234, process)
            .with_error_on_pid(1234);

        // Kill should fail (permission denied)
        let result = mock.kill_process(1234, false);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Simulated error"));

        // Verify kill was NOT recorded
        mock.assert_kill_count(0);
    }

    #[test]
    fn test_error_on_port_not_in_list() {
        // Error port configured but not in the port list - should succeed
        let mock = MockPlatform::new()
            .with_port(3000, 1234)
            .with_error_on_port(8080); // Different port

        let result = mock.get_listening_ports();
        assert!(result.is_ok());
        let ports = result.unwrap();
        assert_eq!(ports.len(), 1);
        assert_eq!(ports[0].port, 3000);
    }

    #[test]
    fn test_error_on_pid_not_in_list() {
        // Error PID configured but different PID queried - should succeed
        let process = ProcessInfo {
            pid: 1234,
            name: "node.exe".to_string(),
            command: "node server.js".to_string(),
            status: ProcessStatus::Healthy,
            memory_kb: 50000,
            cpu_percent: 2.5,
            start_time: None,
            working_dir: None,
        };

        let mock = MockPlatform::new()
            .with_process(1234, process.clone())
            .with_error_on_pid(5678); // Different PID

        // Should succeed for PID 1234
        let result = mock.get_process_info(1234);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), process);

        // Should succeed for kill as well
        let result = mock.kill_process(1234, false);
        assert!(result.is_ok());
        mock.assert_killed(1234);
    }

    #[test]
    fn test_multiple_operations_with_error_simulation() {
        // Test that error simulation affects all relevant operations
        let process = ProcessInfo {
            pid: 1234,
            name: "node.exe".to_string(),
            command: "node server.js".to_string(),
            status: ProcessStatus::Healthy,
            memory_kb: 50000,
            cpu_percent: 2.5,
            start_time: None,
            working_dir: None,
        };

        let tree = vec![ProcessNode {
            pid: 1234,
            ppid: 0,
            name: "node.exe".to_string(),
        }];

        let mock = MockPlatform::new()
            .with_port(3000, 1234)
            .with_process(1234, process)
            .with_process_tree(1234, tree)
            .with_error_on_pid(1234);

        // Port scan succeeds (error is on PID, not port)
        assert!(mock.get_listening_ports().is_ok());

        // All PID-based operations fail
        assert!(mock.get_process_info(1234).is_err());
        assert!(mock.get_process_tree(1234).is_err());
        assert!(mock.kill_process(1234, false).is_err());
    }

    // ========== Property-Based Tests ==========

    use proptest::prelude::*;

    proptest! {
        #[test]
        fn prop_process_chain_depth_limit(
            root_pid in 1000u32..=10000,
            depth in 0usize..=20
        ) {
            // Property: Process chain should be created with correct depth
            let mock = MockPlatform::new().with_process_chain(root_pid, depth);

            if depth == 0 {
                // Empty chain
                assert!(mock.get_process_tree(root_pid).is_err());
            } else {
                let tree = mock.get_process_tree(root_pid).unwrap();
                assert_eq!(tree.len(), depth);

                // Verify chain structure
                for (i, node) in tree.iter().enumerate() {
                    assert_eq!(node.pid, root_pid + i as u32);
                    if i == 0 {
                        assert_eq!(node.ppid, 0); // Root has no parent
                    } else {
                        assert_eq!(node.ppid, root_pid + (i - 1) as u32);
                    }
                }
            }
        }

        #[test]
        fn prop_process_tree_cycle_detection(root_pid in 1000u32..=10000) {
            // Property: Cycle should be detectable in process tree
            let mock = MockPlatform::new().with_process_tree_cycle(root_pid);

            let tree = mock.get_process_tree(root_pid).unwrap();
            assert_eq!(tree.len(), 2);

            // Verify cycle structure
            assert_eq!(tree[0].pid, root_pid);
            assert_eq!(tree[0].ppid, root_pid + 1); // Parent is child
            assert_eq!(tree[1].pid, root_pid + 1);
            assert_eq!(tree[1].ppid, root_pid); // Child's parent is root
        }

        #[test]
        fn prop_process_tree_no_panic(
            root_pid in 1u32..=100000,
            depth in 0usize..=100
        ) {
            // Property: Creating process chains should never panic
            let mock = MockPlatform::new().with_process_chain(root_pid, depth);

            // Should not panic
            let result = mock.get_process_tree(root_pid);

            if depth == 0 {
                assert!(result.is_err());
            } else {
                assert!(result.is_ok());
            }
        }

        #[test]
        fn prop_multiple_process_chains(
            pid1 in 1000u32..=5000,
            pid2 in 6000u32..=10000,
            depth1 in 1usize..=10,
            depth2 in 1usize..=10
        ) {
            // Property: Multiple independent process chains should coexist
            let mock = MockPlatform::new()
                .with_process_chain(pid1, depth1)
                .with_process_chain(pid2, depth2);

            let tree1 = mock.get_process_tree(pid1).unwrap();
            let tree2 = mock.get_process_tree(pid2).unwrap();

            assert_eq!(tree1.len(), depth1);
            assert_eq!(tree2.len(), depth2);

            // Verify trees are independent
            assert_eq!(tree1[0].pid, pid1);
            assert_eq!(tree2[0].pid, pid2);
        }
    }
}
