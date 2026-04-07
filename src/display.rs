use crate::process::{PortInfo, ProcessStatus};
use colored::*;
use serde_json;
use tabled::{Table, Tabled, settings::Style};

/// Display module for formatting output
pub struct Display {
    use_colors: bool,
    json_mode: bool,
}

impl Display {
    pub fn new(use_colors: bool, json_mode: bool) -> Self {
        Self {
            use_colors,
            json_mode,
        }
    }

    /// Display ports in table format
    pub fn show_ports(&self, ports: &[PortInfo]) {
        if self.json_mode {
            self.show_json(ports);
        } else {
            self.show_table(ports);
        }
    }

    fn show_table(&self, ports: &[PortInfo]) {
        if ports.is_empty() {
            println!("No listening ports found.");
            return;
        }

        // Convert to table rows
        let rows: Vec<PortRow> = ports
            .iter()
            .map(|p| PortRow {
                port: p.port.to_string(),
                process: p.process_name.clone(),
                pid: p.pid.to_string(),
                status: self.format_status(p.status),
                project: p.project_name.clone().unwrap_or_else(|| "—".to_string()),
            })
            .collect();

        let mut table = Table::new(rows);
        table.with(Style::rounded());

        println!("{}", table);
    }

    fn show_json(&self, ports: &[PortInfo]) {
        match serde_json::to_string_pretty(ports) {
            Ok(json) => println!("{}", json),
            Err(e) => eprintln!("Error serializing to JSON: {}", e),
        }
    }

    fn format_status(&self, status: ProcessStatus) -> String {
        if !self.use_colors {
            return match status {
                ProcessStatus::Healthy => "● healthy".to_string(),
                ProcessStatus::Orphaned => "● orphaned".to_string(),
                ProcessStatus::Zombie => "● zombie".to_string(),
            };
        }

        match status {
            ProcessStatus::Healthy => format!("{} healthy", "●".green()),
            ProcessStatus::Orphaned => format!("{} orphaned", "●".yellow()),
            ProcessStatus::Zombie => format!("{} zombie", "●".red()),
        }
    }
}

#[derive(Tabled)]
struct PortRow {
    #[tabled(rename = "PORT")]
    port: String,
    #[tabled(rename = "PROCESS")]
    process: String,
    #[tabled(rename = "PID")]
    pid: String,
    #[tabled(rename = "STATUS")]
    status: String,
    #[tabled(rename = "PROJECT")]
    project: String,
}
