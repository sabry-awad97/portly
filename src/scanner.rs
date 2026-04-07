use crate::error::Result;
use crate::platform::Platform;
use crate::process::PortInfo;

/// Port scanner orchestrator
pub struct Scanner {
    platform: Box<dyn Platform>,
}

impl Scanner {
    pub fn new(platform: Box<dyn Platform>) -> Self {
        Self { platform }
    }

    /// Scan for all listening ports
    pub fn scan(&self) -> Result<Vec<PortInfo>> {
        // TODO: Implement port scanning orchestration
        // This will be implemented in Issue #3
        // 1. Get raw port info from platform
        // 2. Enrich with process details
        // 3. Detect frameworks
        // 4. Filter based on config
        Ok(Vec::new())
    }

    /// Get detailed information for a specific port
    pub fn get_port_details(&self, _port: u16) -> Result<PortInfo> {
        // TODO: Implement detailed port lookup
        // This will be implemented in Issue #5
        todo!()
    }
}
