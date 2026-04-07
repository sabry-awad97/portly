use crate::process::PortInfo;
use serde_json;

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

    fn show_table(&self, _ports: &[PortInfo]) {
        // TODO: Implement table display using tabled
        // This will be implemented in Issue #3
        println!("Table display not yet implemented");
    }

    fn show_json(&self, ports: &[PortInfo]) {
        match serde_json::to_string_pretty(ports) {
            Ok(json) => println!("{}", json),
            Err(e) => eprintln!("Error serializing to JSON: {}", e),
        }
    }
}
