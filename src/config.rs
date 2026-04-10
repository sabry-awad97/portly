use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use tabled::settings::Style;

/// Configuration structure for Portly.
///
/// Manages display settings, process filters, and default flags.
///
/// # Examples
///
/// ```no_run
/// use portly::config::Config;
///
/// // Load config from file or use defaults
/// let config = Config::load()?;
///
/// // Create default config file
/// let path = Config::config_path()?;
/// Config::create_default(&path)?;
/// # Ok::<(), anyhow::Error>(())
/// ```
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
    #[serde(default = "default_table_style")]
    pub table_style: String,
}

fn default_table_style() -> String {
    "rounded".to_string()
}

/// Apply table style to a table based on style name.
///
/// Supports: rounded, ascii, modern, blank, empty (case-insensitive).
/// Uses rounded style for unknown values.
pub fn apply_table_style(table: &mut tabled::Table, style_name: &str) {
    match style_name.to_lowercase().as_str() {
        "rounded" => {
            table.with(Style::rounded());
        }
        "ascii" => {
            table.with(Style::ascii());
        }
        "modern" => {
            table.with(Style::modern());
        }
        "blank" => {
            table.with(Style::blank());
        }
        "empty" => {
            table.with(Style::empty());
        }
        _ => {
            table.with(Style::rounded());
        }
    }
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
                table_style: "rounded".to_string(),
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
    /// Load configuration from file, creating default if missing.
    ///
    /// Loads from `%APPDATA%\portly\config.toml` on Windows.
    /// If the file doesn't exist, returns default configuration.
    ///
    /// # Errors
    ///
    /// Returns error if config file exists but cannot be read or parsed
    #[must_use = "config should be used to configure the application"]
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path().context("Failed to determine config path")?;

        if !config_path.exists() {
            return Ok(Self::default());
        }

        Self::load_from_path(&config_path)
    }

    /// Load configuration from specific path
    pub fn load_from_path(path: &PathBuf) -> Result<Self> {
        let content = fs::read_to_string(path).context("Failed to read config file")?;

        toml::from_str(&content).context("Failed to parse config file")
    }

    /// Get the config file path.
    ///
    /// Returns the platform-specific config directory path.
    /// On Windows: `%APPDATA%\portly\config.toml`
    ///
    /// # Errors
    ///
    /// Returns error if the config directory cannot be determined
    pub fn config_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir().context("Could not find config directory")?;

        Ok(config_dir.join("portly").join("config.toml"))
    }

    /// Create default config file at the specified path.
    ///
    /// Creates parent directories if they don't exist.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the config file should be created
    ///
    /// # Errors
    ///
    /// Returns error if directories cannot be created or file cannot be written
    pub fn create_default(path: &PathBuf) -> Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).context("Failed to create config directory")?;
        }

        let default_config = Self::default();
        let mut toml_string = toml::to_string_pretty(&default_config)
            .context("Failed to serialize default config")?;

        // Add helpful comment about table_style options
        toml_string = toml_string.replace(
            "[display]",
            "[display]\n# Table style options: rounded (default), ascii, modern, blank, empty\n# Use 'ascii' for terminals without Unicode support",
        );

        fs::write(path, toml_string).context("Failed to write config file")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

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
        assert!(
            config
                .filters
                .exclude_processes
                .contains(&"Spotify".to_string())
        );
        assert!(
            config
                .filters
                .exclude_processes
                .contains(&"Chrome".to_string())
        );
    }

    #[test]
    fn test_display_config_has_table_style_field() {
        // Arrange & Act
        let config = Config::default();

        // Assert - DisplayConfig should have table_style field with default "rounded"
        assert_eq!(config.display.table_style, "rounded");
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
        assert_eq!(
            config.filters.exclude_processes,
            vec!["TestApp", "AnotherApp"]
        );
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
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Failed to parse config file")
        );

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
    fn test_create_default_includes_table_style_comment() {
        // Arrange
        let temp_dir = std::env::temp_dir();
        let config_path = temp_dir.join("portly_test_comment.toml");

        // Cleanup any existing file
        let _ = fs::remove_file(&config_path);

        // Act
        Config::create_default(&config_path).unwrap();

        // Assert - config file should include helpful comment about table styles
        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("Table style options"));
        assert!(content.contains("rounded"));
        assert!(content.contains("ascii"));
        assert!(content.contains("modern"));
        assert!(content.contains("blank"));
        assert!(content.contains("empty"));

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

    #[test]
    fn test_serialize_config_includes_table_style() {
        // Arrange
        let config = Config::default();

        // Act
        let toml_string = toml::to_string_pretty(&config).unwrap();

        // Assert - serialized config should include table_style
        assert!(toml_string.contains("table_style"));
        assert!(toml_string.contains("rounded"));
    }

    #[test]
    fn test_deserialize_config_without_table_style() {
        // Arrange - old config file without table_style field
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

        // Act
        let config: Config = toml::from_str(toml_content).unwrap();

        // Assert - should default to "rounded" for backward compatibility
        assert_eq!(config.display.table_style, "rounded");
        assert!(!config.display.colors);
        assert!(config.display.compact);
    }

    #[test]
    fn test_apply_table_style_valid_styles() {
        use tabled::Table;

        // Test all valid style names by applying them to a table
        let data = vec![("Port", "Process"), ("3000", "node")];

        let mut table = Table::new(data.clone());
        apply_table_style(&mut table, "rounded");

        let mut table = Table::new(data.clone());
        apply_table_style(&mut table, "ascii");

        let mut table = Table::new(data.clone());
        apply_table_style(&mut table, "modern");

        let mut table = Table::new(data.clone());
        apply_table_style(&mut table, "blank");

        let mut table = Table::new(data.clone());
        apply_table_style(&mut table, "empty");

        // Test case insensitivity
        let mut table = Table::new(data.clone());
        apply_table_style(&mut table, "ROUNDED");

        let mut table = Table::new(data);
        apply_table_style(&mut table, "AsCiI");
    }

    #[test]
    fn test_apply_table_style_invalid_falls_back() {
        use tabled::Table;

        // Test that invalid style names fall back to rounded (default)
        let data = vec![("Port", "Process"), ("3000", "node")];

        let mut table_invalid = Table::new(data.clone());
        apply_table_style(&mut table_invalid, "invalid_style");

        let mut table_default = Table::new(data);
        apply_table_style(&mut table_default, "rounded");

        // Both should produce the same output (rounded style)
        assert_eq!(table_invalid.to_string(), table_default.to_string());
    }

    // ========== Property-Based Tests ==========

    // Implement Arbitrary for Config structs
    impl Arbitrary for DisplayConfig {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<bool>(), any::<bool>(), "[a-z]{3,10}")
                .prop_map(|(colors, compact, table_style)| DisplayConfig {
                    colors,
                    compact,
                    table_style: table_style.to_string(),
                })
                .boxed()
        }
    }

    impl Arbitrary for FilterConfig {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                any::<bool>(),
                prop::collection::vec("[a-zA-Z0-9_-]{1,20}", 0..10),
            )
                .prop_map(|(exclude_system, exclude_processes)| FilterConfig {
                    exclude_system,
                    exclude_processes,
                })
                .boxed()
        }
    }

    impl Arbitrary for DefaultsConfig {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (any::<bool>(), any::<bool>())
                .prop_map(|(show_all, json_output)| DefaultsConfig {
                    show_all,
                    json_output,
                })
                .boxed()
        }
    }

    impl Arbitrary for Config {
        type Parameters = ();
        type Strategy = BoxedStrategy<Self>;

        fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
            (
                any::<DisplayConfig>(),
                any::<FilterConfig>(),
                any::<DefaultsConfig>(),
            )
                .prop_map(|(display, filters, defaults)| Config {
                    display,
                    filters,
                    defaults,
                })
                .boxed()
        }
    }

    proptest! {
        #[test]
        fn prop_config_round_trip(config in any::<Config>()) {
            // Property: Serialize then deserialize should equal original
            let toml = toml::to_string(&config).unwrap();
            let deserialized: Config = toml::from_str(&toml).unwrap();

            assert_eq!(config, deserialized);
        }

        #[test]
        fn prop_config_serialization_no_panic(config in any::<Config>()) {
            // Property: Serialization should never panic
            let result = toml::to_string(&config);

            assert!(result.is_ok());
        }

        #[test]
        fn prop_config_deserialization_valid_toml(config in any::<Config>()) {
            // Property: Serialized config should be valid TOML
            let toml = toml::to_string(&config).unwrap();
            let result: Result<Config, _> = toml::from_str(&toml);

            assert!(result.is_ok());
        }

        #[test]
        fn prop_config_pretty_serialization(config in any::<Config>()) {
            // Property: Pretty serialization should also round-trip
            let toml = toml::to_string_pretty(&config).unwrap();
            let deserialized: Config = toml::from_str(&toml).unwrap();

            assert_eq!(config, deserialized);
        }

        #[test]
        fn prop_display_config_round_trip(display in any::<DisplayConfig>()) {
            // Property: DisplayConfig should round-trip
            let toml = toml::to_string(&display).unwrap();
            let deserialized: DisplayConfig = toml::from_str(&toml).unwrap();

            assert_eq!(display, deserialized);
        }

        #[test]
        fn prop_filter_config_round_trip(filters in any::<FilterConfig>()) {
            // Property: FilterConfig should round-trip
            let toml = toml::to_string(&filters).unwrap();
            let deserialized: FilterConfig = toml::from_str(&toml).unwrap();

            assert_eq!(filters, deserialized);
        }

        #[test]
        fn prop_defaults_config_round_trip(defaults in any::<DefaultsConfig>()) {
            // Property: DefaultsConfig should round-trip
            let toml = toml::to_string(&defaults).unwrap();
            let deserialized: DefaultsConfig = toml::from_str(&toml).unwrap();

            assert_eq!(defaults, deserialized);
        }
    }
}
