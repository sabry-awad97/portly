use crate::cli::ConfigAction;
use crate::config;
use anyhow::Context;

pub fn handle_config(action: ConfigAction) -> anyhow::Result<()> {
    match action {
        ConfigAction::Init => {
            let config_path =
                config::Config::config_path().context("Failed to determine config path")?;

            if config_path.exists() {
                println!("Config file already exists at: {}", config_path.display());
                println!("Use 'portly config reset' to restore defaults.");
                return Ok(());
            }

            config::Config::create_default(&config_path).context("Failed to create config file")?;

            println!("✓ Created default config at: {}", config_path.display());
            println!("\nEdit this file to customize:");
            println!("  - Excluded processes");
            println!("  - Color schemes");
            println!("  - Default flags");
            Ok(())
        }
        ConfigAction::Path => {
            let config_path =
                config::Config::config_path().context("Failed to determine config path")?;

            if config_path.exists() {
                println!("{}", config_path.display());
            } else {
                println!("{} (not created yet)", config_path.display());
                println!("\nRun 'portly config init' to create it.");
            }
            Ok(())
        }
        ConfigAction::Reset => {
            let config_path =
                config::Config::config_path().context("Failed to determine config path")?;

            config::Config::create_default(&config_path).context("Failed to reset config file")?;

            println!("✓ Reset config to defaults at: {}", config_path.display());
            Ok(())
        }
    }
}
