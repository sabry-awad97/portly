use serde::{Deserialize, Serialize};

/// Configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub display: DisplayConfig,
    pub filters: FilterConfig,
    pub defaults: DefaultsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisplayConfig {
    pub colors: bool,
    pub compact: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterConfig {
    pub exclude_system: bool,
    pub exclude_processes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefaultsConfig {
    pub show_all: bool,
    pub json_output: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            display: DisplayConfig {
                colors: true,
                compact: false,
            },
            filters: FilterConfig {
                exclude_system: true,
                exclude_processes: vec![
                    "Spotify".to_string(),
                    "Chrome".to_string(),
                    "Firefox".to_string(),
                    "Slack".to_string(),
                    "Discord".to_string(),
                    "Code".to_string(),
                    "Teams".to_string(),
                ],
            },
            defaults: DefaultsConfig {
                show_all: false,
                json_output: false,
            },
        }
    }
}

impl Config {
    /// Load configuration from file
    pub fn load() -> anyhow::Result<Self> {
        // TODO: Implement config file loading
        // This will be implemented in Issue #11
        Ok(Self::default())
    }
}
