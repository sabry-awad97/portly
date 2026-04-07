use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Configuration structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Config {
    pub display: DisplayConfig,
    pub filters: FilterConfig,
    pub defaults: DefaultsConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DisplayConfig {
    pub colors: bool,
    pub compact: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FilterConfig {
    pub exclude_system: bool,
    pub exclude_processes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    /// Load configuration from file, creating default if missing
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()
            .context("Failed to determine config path")?;
        
        if !config_path.exists() {
            return Ok(Self::default());
        }
        
        Self::load_from_path(&config_path)
    }
    
    /// Load configuration from specific path
    pub fn load_from_path(path: &PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path)
            .context("Failed to read config file")?;
        
        toml::from_str(&content)
            .context("Failed to parse config file")
    }
    
    /// Get the config file path
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not find config directory")?;
        
        Ok(config_dir.join("portly").join("config.toml"))
    }
    
    /// Create default config file
    pub fn create_default(path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .context("Failed to create config directory")?;
        }
        
        let default_config = Self::default();
        let toml_string = toml::to_string_pretty(&default_config)
            .context("Failed to serialize default config")?;
        
        fs::write(path, toml_string)
            .context("Failed to write config file")?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_default_config_has_expected_values() {
        // Arrange & Act
        let config = Config::default();
        
        // Assert
        assert!(config.display.colors);
        assert!(!config.display.compact);
        assert!(config.filters.exclude_system);
        assert!(!config.defaults.show_all);
        assert!(!config.defaults.json_output);
        assert!(config.filters.exclude_processes.contains(&"Spotify".to_string()));
        assert!(config.filters.exclude_processes.contains(&"Chrome".to_string()));
    }
    
    #[test]
    fn test_load_returns_default_when_file_missing() {
        // This test verifies behavior when config file doesn't exist
        // We can't easily test the actual load() because it uses real paths
        // Instead we test that default config is valid
        let config = Config::default();
        assert_eq!(config.filters.exclude_processes.len(), 7);
    }
    
    #[test]
    fn test_parse_valid_toml_config() {
        // Arrange
        let toml_content = r#"
[display]
colors = false
compact = true

[filters]
exclude_system = false
exclude_processes = ["TestApp", "AnotherApp"]

[defaults]
show_all = true
json_output = true
"#;
        
        // Act
        let config: Config = toml::from_str(toml_content).unwrap();
        
        // Assert
        assert!(!config.display.colors);
        assert!(config.display.compact);
        assert!(!config.filters.exclude_system);
        assert_eq!(config.filters.exclude_processes, vec!["TestApp", "AnotherApp"]);
        assert!(config.defaults.show_all);
        assert!(config.defaults.json_output);
    }
    
    #[test]
    fn test_load_from_path_with_valid_file() {
        // Arrange
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("portly_test_config.toml");
        
        let toml_content = r#"
[display]
colors = false
compact = true

[filters]
exclude_system = false
exclude_processes = ["TestApp"]

[defaults]
show_all = true
json_output = false
"#;
        
        fs::write(&config_path, toml_content).unwrap();
        
        // Act
        let config = Config::load_from_path(&config_path).unwrap();
        
        // Assert
        assert!(!config.display.colors);
        assert!(config.display.compact);
        assert_eq!(config.filters.exclude_processes, vec!["TestApp"]);
        
        // Cleanup
        let _ = fs::remove_file(&config_path);
    }
    
    #[test]
    fn test_load_from_path_with_invalid_toml() {
        // Arrange
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("portly_test_invalid.toml");
        
        let invalid_toml = r#"
[display
colors = false
"#;
        
        fs::write(&config_path, invalid_toml).unwrap();
        
        // Act
        let result = Config::load_from_path(&config_path);
        
        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Failed to parse config file"));
        
        // Cleanup
        let _ = fs::remove_file(&config_path);
    }
    
    #[test]
    fn test_create_default_config_file() {
        // Arrange
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("portly_test_default.toml");
        
        // Cleanup any existing file
        let _ = fs::remove_file(&config_path);
        
        // Act
        Config::create_default(&config_path).unwrap();
        
        // Assert
        assert!(config_path.exists());
        
        let loaded_config = Config::load_from_path(&config_path).unwrap();
        let default_config = Config::default();
        assert_eq!(loaded_config, default_config);
        
        // Cleanup
        let _ = fs::remove_file(&config_path);
    }
    
    #[test]
    fn test_serialize_config_to_toml() {
        // Arrange
        let config = Config::default();
        
        // Act
        let toml_string = toml::to_string_pretty(&config).unwrap();
        
        // Assert
        assert!(toml_string.contains("[display]"));
        assert!(toml_string.contains("colors = true"));
        assert!(toml_string.contains("[filters]"));
        assert!(toml_string.contains("exclude_system = true"));
        assert!(toml_string.contains("Spotify"));
    }
}

/// Runtime configuration after merging config file with CLI flags
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeConfig {
    pub colors: bool,
    pub compact: bool,
    pub exclude_system: bool,
    pub exclude_processes: Vec<String>,
    pub show_all: bool,
    pub json_output: bool,
}

impl RuntimeConfig {
    /// Merge config file with CLI flags (CLI takes precedence)
    pub fn from_config_and_cli(
        config: Config,
        cli_colors: Option<bool>,
        cli_compact: Option<bool>,
        cli_show_all: Option<bool>,
        cli_json: Option<bool>,
    ) -> Self {
        Self {
            colors: cli_colors.unwrap_or(config.display.colors),
            compact: cli_compact.unwrap_or(config.display.compact),
            exclude_system: config.filters.exclude_system,
            exclude_processes: config.filters.exclude_processes,
            show_all: cli_show_all.unwrap_or(config.defaults.show_all),
            json_output: cli_json.unwrap_or(config.defaults.json_output),
        }
    }
}

#[cfg(test)]
mod runtime_tests {
    use super::*;
    
    #[test]
    fn test_runtime_config_uses_config_defaults_when_no_cli_flags() {
        // Arrange
        let config = Config::default();
        
        // Act
        let runtime = RuntimeConfig::from_config_and_cli(
            config.clone(),
            None,
            None,
            None,
            None,
        );
        
        // Assert
        assert_eq!(runtime.colors, config.display.colors);
        assert_eq!(runtime.compact, config.display.compact);
        assert_eq!(runtime.show_all, config.defaults.show_all);
        assert_eq!(runtime.json_output, config.defaults.json_output);
    }
    
    #[test]
    fn test_runtime_config_cli_flags_override_config() {
        // Arrange
        let mut config = Config::default();
        config.display.colors = true;
        config.display.compact = false;
        config.defaults.show_all = false;
        config.defaults.json_output = false;
        
        // Act - CLI flags override config
        let runtime = RuntimeConfig::from_config_and_cli(
            config,
            Some(false),  // Override colors
            Some(true),   // Override compact
            Some(true),   // Override show_all
            Some(true),   // Override json
        );
        
        // Assert - CLI flags take precedence
        assert!(!runtime.colors);
        assert!(runtime.compact);
        assert!(runtime.show_all);
        assert!(runtime.json_output);
    }
    
    #[test]
    fn test_runtime_config_partial_cli_overrides() {
        // Arrange
        let config = Config::default();
        
        // Act - Only some CLI flags provided
        let runtime = RuntimeConfig::from_config_and_cli(
            config.clone(),
            Some(false),  // Override colors only
            None,
            None,
            None,
        );
        
        // Assert
        assert!(!runtime.colors);  // Overridden
        assert_eq!(runtime.compact, config.display.compact);  // From config
        assert_eq!(runtime.show_all, config.defaults.show_all);  // From config
        assert_eq!(runtime.json_output, config.defaults.json_output);  // From config
    }
    
    #[test]
    fn test_runtime_config_preserves_exclude_processes() {
        // Arrange
        let config = Config::default();
        let expected_processes = config.filters.exclude_processes.clone();
        
        // Act
        let runtime = RuntimeConfig::from_config_and_cli(
            config,
            None,
            None,
            None,
            None,
        );
        
        // Assert
        assert_eq!(runtime.exclude_processes, expected_processes);
        assert!(runtime.exclude_processes.contains(&"Spotify".to_string()));
    }
}
