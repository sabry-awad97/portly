use serde::{Deserialize, Serialize};

/// Process status indicator
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessStatus {
    Healthy,
    Orphaned,
    Zombie,
}

/// Basic process information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub command: String,
    pub status: ProcessStatus,
    pub memory_kb: u64,
    pub cpu_percent: f32,
    pub start_time: Option<std::time::SystemTime>,
    pub working_dir: Option<String>,
}

/// Process tree node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessNode {
    pub pid: u32,
    pub ppid: u32,
    pub name: String,
}

/// Raw port information from platform
#[derive(Debug, Clone)]
pub struct RawPortInfo {
    pub port: u16,
    pub pid: u32,
}

/// Enriched port information with process details
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PortInfo {
    pub port: u16,
    pub pid: u32,
    pub process_name: String,
    pub status: ProcessStatus,
    pub framework: Option<String>,
    pub project_name: Option<String>,
}
