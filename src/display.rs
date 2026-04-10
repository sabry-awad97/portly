use crate::process::{PortInfo, ProcessStatus};
use colored::*;
use tabled::{Table, Tabled, settings::Style};

/// Display formatter for port information.
///
/// Handles both table and JSON output formatting with optional colors.
///
/// # Examples
///
/// ```no_run
/// use portly::display::Display;
///
/// let display = Display::new(true, false); // colors enabled, not JSON
/// display.show_ports(&ports);
/// ```
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
                framework: self.format_framework(p.framework.as_deref()),
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

    fn format_framework(&self, framework: Option<&str>) -> String {
        let framework_str = framework.unwrap_or("—");
        crate::colors::apply_framework_color(framework_str, self.use_colors)
    }

    /// Apply color to text if colors are enabled
    #[allow(dead_code)]
    fn colorize(&self, text: &str, color: Color) -> String {
        if self.use_colors {
            text.color(color).to_string()
        } else {
            text.to_string()
        }
    }

    /// Green "▲ NEW   " marker for watch events
    fn new_marker(&self) -> String {
        self.colorize("▲ NEW   ", Color::Green)
    }

    /// Red "▼ CLOSED" marker for watch events
    fn closed_marker(&self) -> String {
        self.colorize("▼ CLOSED", Color::Red)
    }

    /// Green "✓" marker for success
    #[allow(dead_code)]
    fn success_marker(&self) -> String {
        self.colorize("✓", Color::Green)
    }

    /// Red "✗" marker for errors
    #[allow(dead_code)]
    fn error_marker(&self) -> String {
        self.colorize("✗", Color::Red)
    }

    /// Display a new port event in watch mode
    pub fn show_watch_event_new(&self, port_info: &PortInfo) {
        let timestamp = chrono::Local::now().format("%H:%M:%S");
        let event_marker = self.new_marker();

        let framework_str = port_info
            .framework
            .as_ref()
            .map(|f| format!(" {}", f))
            .unwrap_or_default();

        let project_str = port_info
            .project_name
            .as_ref()
            .map(|p| format!(" [{}]", p))
            .unwrap_or_default();

        println!(
            "{} {} :{} ← {}{}{}",
            timestamp,
            event_marker,
            port_info.port,
            port_info.process_name,
            project_str,
            framework_str
        );
    }

    /// Display a closed port event in watch mode
    pub fn show_watch_event_closed(&self, port: u16) {
        let timestamp = chrono::Local::now().format("%H:%M:%S");
        let event_marker = self.closed_marker();

        println!("{} {} :{}", timestamp, event_marker, port);
    }

    /// Display process table for ps command
    pub fn show_ps_table(&self, processes: &[crate::commands::ps::PsProcess]) {
        if processes.is_empty() {
            println!("No processes found.");
            return;
        }

        #[derive(Tabled)]
        struct PsRow {
            #[tabled(rename = "PID")]
            pid: String,
            #[tabled(rename = "PROCESS")]
            process: String,
            #[tabled(rename = "CPU%")]
            cpu: String,
            #[tabled(rename = "MEM")]
            mem: String,
            #[tabled(rename = "PROJECT")]
            project: String,
            #[tabled(rename = "FRAMEWORK")]
            framework: String,
            #[tabled(rename = "UPTIME")]
            uptime: String,
            #[tabled(rename = "WHAT")]
            what: String,
        }

        let rows: Vec<PsRow> = processes
            .iter()
            .map(|p| {
                let cpu_str = format!("{:.1}", p.cpu_percent);
                let cpu_colored = self.format_cpu_percent(p.cpu_percent, &cpu_str);

                let mem_str = crate::details::format_memory(p.memory_kb);

                PsRow {
                    pid: p.pid.to_string(),
                    process: p.name.clone(),
                    cpu: cpu_colored,
                    mem: mem_str,
                    project: p.project_name.clone().unwrap_or_else(|| "—".to_string()),
                    framework: p.framework.clone().unwrap_or_else(|| "—".to_string()),
                    uptime: p.uptime.clone(),
                    what: p.what.clone(),
                }
            })
            .collect();

        let mut table = Table::new(rows);
        table.with(Style::rounded());
        println!("{}", table);
    }

    /// Format CPU percentage with color coding
    fn format_cpu_percent(&self, cpu: f32, cpu_str: &str) -> String {
        if !self.use_colors {
            return cpu_str.to_string();
        }

        use colored::Colorize;
        if cpu > 25.0 {
            cpu_str.red().to_string()
        } else if cpu > 5.0 {
            cpu_str.yellow().to_string()
        } else {
            cpu_str.green().to_string()
        }
    }

    /// Format uptime as human-readable duration
    pub fn format_uptime(&self, start_time: Option<std::time::SystemTime>) -> String {
        let Some(start) = start_time else {
            return "—".to_string();
        };

        let Ok(duration) = std::time::SystemTime::now().duration_since(start) else {
            return "—".to_string();
        };

        let total_secs = duration.as_secs();
        let days = total_secs.saturating_div(86400);
        let hours = (total_secs % 86400).saturating_div(3600);
        let minutes = (total_secs % 3600).saturating_div(60);

        if days > 0 {
            format!("{}d {}h", days, hours)
        } else if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        }
    }

    /// Extract concise command description from full command line
    pub fn format_command(&self, cmd_line: &str, process_name: &str) -> String {
        if cmd_line.is_empty() {
            return process_name.to_string();
        }

        let parts: Vec<&str> = cmd_line.split_whitespace().collect();
        if parts.is_empty() {
            return process_name.to_string();
        }

        let first = parts[0].to_lowercase();

        // Node.js: "node /path/to/next dev" → "next dev"
        if first.contains("node") && parts.len() > 1 {
            let rest: Vec<&str> = parts[1..]
                .iter()
                .skip_while(|p| p.starts_with('-'))
                .copied()
                .collect();

            if !rest.is_empty() {
                let first_arg = rest[0];
                let cmd_name = std::path::Path::new(first_arg)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| first_arg.to_string());

                if rest.len() > 1 {
                    let desc = format!("{} {}", cmd_name, rest[1..].join(" "));
                    return self.truncate(&desc, 30);
                } else {
                    return self.truncate(&cmd_name, 30);
                }
            }
        }

        // Python: "python manage.py runserver" → "manage.py runserver"
        if first.contains("python") && parts.len() > 2 {
            let desc = parts[2..].join(" ");
            return self.truncate(&desc, 30);
        }

        // Cargo: "cargo run --bin server" → "run --bin server"
        if first.contains("cargo") && parts.len() > 1 {
            let desc = parts[1..].join(" ");
            return self.truncate(&desc, 30);
        }

        // Docker: show container count from command
        if first.contains("docker") {
            return self.truncate(cmd_line, 30);
        }

        // Default: return process name
        self.truncate(process_name, 30)
    }

    /// Truncate string to max length with ellipsis
    fn truncate(&self, text: &str, max_len: usize) -> String {
        if text.len() <= max_len {
            text.to_string()
        } else {
            format!("{}...", &text[..max_len - 3])
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
    #[tabled(rename = "FRAMEWORK")]
    framework: String,
    #[tabled(rename = "STATUS")]
    status: String,
    #[tabled(rename = "PROJECT")]
    project: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::SystemTime;

    #[test]
    fn test_colorize_with_colors() {
        // Force enable colors for this test
        colored::control::set_override(true);
        
        let display = Display::new(true, false);
        let result = display.colorize("test", Color::Green);
        // Should contain ANSI color codes
        assert!(result.contains("\x1b["));
        
        // Reset color override
        colored::control::unset_override();
    }

    #[test]
    fn test_colorize_without_colors() {
        let display = Display::new(false, false);
        let result = display.colorize("test", Color::Green);
        assert_eq!(result, "test");
    }

    #[test]
    fn test_new_marker_with_colors() {
        // Force enable colors for this test
        colored::control::set_override(true);
        
        let display = Display::new(true, false);
        let marker = display.new_marker();
        assert!(marker.contains("▲ NEW"));
        assert!(marker.contains("\x1b[")); // ANSI codes
        
        // Reset color override
        colored::control::unset_override();
    }

    #[test]
    fn test_new_marker_without_colors() {
        let display = Display::new(false, false);
        assert_eq!(display.new_marker(), "▲ NEW   ");
    }

    #[test]
    fn test_closed_marker_with_colors() {
        // Force enable colors for this test
        colored::control::set_override(true);
        
        let display = Display::new(true, false);
        let marker = display.closed_marker();
        assert!(marker.contains("▼ CLOSED"));
        assert!(marker.contains("\x1b["));
        
        // Reset color override
        colored::control::unset_override();
    }

    #[test]
    fn test_closed_marker_without_colors() {
        let display = Display::new(false, false);
        assert_eq!(display.closed_marker(), "▼ CLOSED");
    }

    #[test]
    fn test_success_marker_with_colors() {
        // Force enable colors for this test
        colored::control::set_override(true);
        
        let display = Display::new(true, false);
        let marker = display.success_marker();
        assert!(marker.contains("✓"));
        assert!(marker.contains("\x1b["));
        
        // Reset color override
        colored::control::unset_override();
    }

    #[test]
    fn test_success_marker_without_colors() {
        let display = Display::new(false, false);
        assert_eq!(display.success_marker(), "✓");
    }

    #[test]
    fn test_error_marker_with_colors() {
        // Force enable colors for this test
        colored::control::set_override(true);
        
        let display = Display::new(true, false);
        let marker = display.error_marker();
        assert!(marker.contains("✗"));
        assert!(marker.contains("\x1b["));
        
        // Reset color override
        colored::control::unset_override();
    }

    #[test]
    fn test_error_marker_without_colors() {
        let display = Display::new(false, false);
        assert_eq!(display.error_marker(), "✗");
    }

    #[test]
    fn test_show_watch_event_new_basic() {
        let display = Display::new(false, false);
        let port_info = PortInfo {
            port: 3000,
            pid: 1234,
            process_name: "node".to_string(),
            framework: None,
            project_name: None,
            status: ProcessStatus::Healthy,
        };

        // This will print to stdout - manual verification or capture needed
        display.show_watch_event_new(&port_info);
        // Expected output: "[HH:MM:SS] ▲ NEW    :3000 ← node"
    }

    #[test]
    fn test_show_watch_event_new_with_framework() {
        let display = Display::new(false, false);
        let port_info = PortInfo {
            port: 3000,
            pid: 1234,
            process_name: "node".to_string(),
            framework: Some("Next.js".to_string()),
            project_name: Some("my-app".to_string()),
            status: ProcessStatus::Healthy,
        };

        display.show_watch_event_new(&port_info);
        // Expected output: "[HH:MM:SS] ▲ NEW    :3000 ← node [my-app] Next.js"
    }

    #[test]
    fn test_show_watch_event_closed() {
        let display = Display::new(false, false);
        display.show_watch_event_closed(3000);
        // Expected output: "[HH:MM:SS] ▼ CLOSED :3000"
    }

    #[test]
    fn test_format_cpu_percent_high() {
        let display = Display::new(false, false);
        let result = display.format_cpu_percent(30.0, "30.0");
        assert_eq!(result, "30.0"); // no color
    }

    #[test]
    fn test_format_cpu_percent_medium() {
        let display = Display::new(false, false);
        let result = display.format_cpu_percent(10.0, "10.0");
        assert_eq!(result, "10.0");
    }

    #[test]
    fn test_format_cpu_percent_low() {
        let display = Display::new(false, false);
        let result = display.format_cpu_percent(2.0, "2.0");
        assert_eq!(result, "2.0");
    }

    #[test]
    fn test_format_cpu_percent_with_colors() {
        // Force enable colors for this test
        colored::control::set_override(true);
        
        let display = Display::new(true, false);
        let high = display.format_cpu_percent(30.0, "30.0");
        let medium = display.format_cpu_percent(10.0, "10.0");
        let low = display.format_cpu_percent(2.0, "2.0");

        // Should contain ANSI color codes
        assert!(high.contains("\x1b[")); // red
        assert!(medium.contains("\x1b[")); // yellow
        assert!(low.contains("\x1b[")); // green
        
        // Reset color override
        colored::control::unset_override();
    }

    #[test]
    fn test_show_ps_table_empty() {
        let display = Display::new(false, false);
        let processes: Vec<crate::commands::ps::PsProcess> = vec![];

        // Should print "No processes found."
        display.show_ps_table(&processes);
    }

    #[test]
    fn test_format_uptime_none() {
        let display = Display::new(false, false);
        assert_eq!(display.format_uptime(None), "—");
    }

    #[test]
    fn test_format_uptime_minutes() {
        let display = Display::new(false, false);
        let start = std::time::SystemTime::now() - std::time::Duration::from_secs(300); // 5 minutes ago
        let result = display.format_uptime(Some(start));
        assert_eq!(result, "5m");
    }

    #[test]
    fn test_format_uptime_hours() {
        let display = Display::new(false, false);
        let start = std::time::SystemTime::now() - std::time::Duration::from_secs(7200); // 2 hours ago
        let result = display.format_uptime(Some(start));
        assert_eq!(result, "2h 0m");
    }

    #[test]
    fn test_format_uptime_days() {
        let display = Display::new(false, false);
        let start = std::time::SystemTime::now() - std::time::Duration::from_secs(90000); // 1 day 1 hour ago
        let result = display.format_uptime(Some(start));
        assert_eq!(result, "1d 1h");
    }

    #[test]
    fn test_format_command_node() {
        let display = Display::new(false, false);
        let result = display.format_command("node /path/to/next dev", "node");
        assert_eq!(result, "next dev");
    }

    #[test]
    fn test_format_command_python() {
        let display = Display::new(false, false);
        let result = display.format_command("python manage.py runserver", "python");
        assert_eq!(result, "runserver");
    }

    #[test]
    fn test_format_command_cargo() {
        let display = Display::new(false, false);
        let result = display.format_command("cargo run --bin server", "cargo");
        assert_eq!(result, "run --bin server");
    }

    #[test]
    fn test_format_command_empty() {
        let display = Display::new(false, false);
        let result = display.format_command("", "myprocess");
        assert_eq!(result, "myprocess");
    }

    #[test]
    fn test_truncate_short() {
        let display = Display::new(false, false);
        let result = display.truncate("short", 10);
        assert_eq!(result, "short");
    }

    #[test]
    fn test_truncate_long() {
        let display = Display::new(false, false);
        let result = display.truncate("this is a very long string", 10);
        assert_eq!(result, "this is...");
    }

    #[test]
    fn test_truncate_exact() {
        let display = Display::new(false, false);
        let result = display.truncate("exactly10c", 10);
        assert_eq!(result, "exactly10c");
    }

    // ========== Snapshot Tests ==========

    #[test]
    fn snapshot_format_uptime_variations() {
        let display = Display::new(false, false);
        let now = SystemTime::now();

        // Test various durations
        let five_min = now - std::time::Duration::from_secs(300);
        let thirty_min = now - std::time::Duration::from_secs(1800);
        let two_hours = now - std::time::Duration::from_secs(7200);
        let one_day = now - std::time::Duration::from_secs(90000);
        let five_days = now - std::time::Duration::from_secs(432000);

        let results = format!(
            "5 minutes: {}\n30 minutes: {}\n2 hours: {}\n1 day: {}\n5 days: {}\nNone: {}",
            display.format_uptime(Some(five_min)),
            display.format_uptime(Some(thirty_min)),
            display.format_uptime(Some(two_hours)),
            display.format_uptime(Some(one_day)),
            display.format_uptime(Some(five_days)),
            display.format_uptime(None)
        );

        insta::assert_snapshot!("uptime_variations", results);
    }

    #[test]
    fn snapshot_format_command_patterns() {
        let display = Display::new(false, false);

        let test_cases = vec![
            ("node /path/to/next dev", "node", "Node.js with next dev"),
            (
                "node server.js --port 3000",
                "node",
                "Node.js with server.js",
            ),
            ("python manage.py runserver", "python", "Python Django"),
            ("python -m flask run", "python", "Python Flask"),
            ("cargo run --bin server", "cargo", "Cargo run"),
            ("cargo watch -x run", "cargo", "Cargo watch"),
            ("docker-compose up", "docker-compose", "Docker compose"),
            ("", "myprocess", "Empty command"),
            (
                "very-long-command-that-should-be-truncated-because-it-exceeds-limit",
                "proc",
                "Long command",
            ),
        ];

        let mut results = String::new();
        for (cmd, proc, desc) in test_cases {
            let formatted = display.format_command(cmd, proc);
            results.push_str(&format!("{}: {}\n", desc, formatted));
        }

        insta::assert_snapshot!("command_patterns", results);
    }

    #[test]
    fn snapshot_color_markers_no_color() {
        let display = Display::new(false, false);

        let markers = format!(
            "New: {}\nClosed: {}\nSuccess: {}\nError: {}",
            display.new_marker(),
            display.closed_marker(),
            display.success_marker(),
            display.error_marker()
        );

        insta::assert_snapshot!("markers_no_color", markers);
    }

    #[test]
    fn snapshot_truncate_edge_cases() {
        let display = Display::new(false, false);

        let test_cases = vec![
            ("short", 10, "Short string"),
            ("exactly10c", 10, "Exact length"),
            (
                "this is a very long string that needs truncation",
                10,
                "Long string",
            ),
            ("", 10, "Empty string"),
            ("a", 10, "Single char"),
            ("unicode: 你好世界", 15, "Unicode chars"),
        ];

        let mut results = String::new();
        for (text, max_len, desc) in test_cases {
            let truncated = display.truncate(text, max_len);
            results.push_str(&format!("{}: '{}'\n", desc, truncated));
        }

        insta::assert_snapshot!("truncate_edge_cases", results);
    }

    #[test]
    fn snapshot_format_status_no_color() {
        let display = Display::new(false, false);

        let statuses = format!(
            "Healthy: {}\nOrphaned: {}\nZombie: {}",
            display.format_status(ProcessStatus::Healthy),
            display.format_status(ProcessStatus::Orphaned),
            display.format_status(ProcessStatus::Zombie)
        );

        insta::assert_snapshot!("status_no_color", statuses);
    }

    #[test]
    fn snapshot_format_framework_no_color() {
        let display = Display::new(false, false);

        let frameworks = vec![
            Some("Next.js"),
            Some("Django"),
            Some("Rails"),
            Some("Rust"),
            Some("PostgreSQL"),
            Some("Docker"),
            Some("Unknown Framework"),
            None,
        ];

        let mut results = String::new();
        for fw in frameworks {
            let formatted = display.format_framework(fw);
            results.push_str(&format!("{:?}: {}\n", fw, formatted));
        }

        insta::assert_snapshot!("framework_no_color", results);
    }

    #[test]
    fn test_display_uses_shared_colors() {
        // Verify that display.rs uses the shared colors module
        colored::control::set_override(true);
        
        let display = Display::new(true, false);
        
        // Test a few frameworks to ensure they match the shared color module
        let frameworks = vec!["Next.js", "Django", "Rust", "PostgreSQL", "Docker"];
        
        for framework in frameworks {
            let display_result = display.format_framework(Some(framework));
            let shared_result = crate::colors::apply_framework_color(framework, true);
            
            assert_eq!(display_result, shared_result, 
                "Display formatting for {} should match shared colors module", framework);
        }
        
        colored::control::unset_override();
    }

    #[test]
    fn snapshot_format_cpu_percent_no_color() {
        let display = Display::new(false, false);

        let test_cases = vec![
            (0.5, "0.5", "Very low"),
            (2.0, "2.0", "Low"),
            (5.0, "5.0", "Medium low"),
            (10.0, "10.0", "Medium"),
            (25.0, "25.0", "High threshold"),
            (50.0, "50.0", "High"),
            (99.9, "99.9", "Very high"),
        ];

        let mut results = String::new();
        for (cpu, cpu_str, desc) in test_cases {
            let formatted = display.format_cpu_percent(cpu, cpu_str);
            results.push_str(&format!("{}: {}\n", desc, formatted));
        }

        insta::assert_snapshot!("cpu_percent_no_color", results);
    }

    #[test]
    fn snapshot_show_ports_json_mode() {
        let _display = Display::new(false, true);

        let ports = vec![
            PortInfo {
                port: 3000,
                pid: 1234,
                process_name: "node".to_string(),
                framework: Some("Next.js".to_string()),
                project_name: Some("my-app".to_string()),
                status: ProcessStatus::Healthy,
            },
            PortInfo {
                port: 5000,
                pid: 5678,
                process_name: "python".to_string(),
                framework: Some("Django".to_string()),
                project_name: Some("api-server".to_string()),
                status: ProcessStatus::Orphaned,
            },
            PortInfo {
                port: 8080,
                pid: 9999,
                process_name: "java".to_string(),
                framework: None,
                project_name: None,
                status: ProcessStatus::Zombie,
            },
        ];

        // Capture JSON output
        let json_output = serde_json::to_string_pretty(&ports).unwrap();
        insta::assert_snapshot!("ports_json", json_output);
    }

    #[test]
    fn snapshot_show_ports_empty_json() {
        let ports: Vec<PortInfo> = vec![];
        let json_output = serde_json::to_string_pretty(&ports).unwrap();
        insta::assert_snapshot!("ports_empty_json", json_output);
    }

    #[test]
    fn snapshot_format_command_special_characters() {
        let display = Display::new(false, false);

        let test_cases = vec![
            (
                "node app.js --env=\"production\"",
                "node",
                "Quotes in command",
            ),
            (
                "python script.py --path=/home/user/my project",
                "python",
                "Spaces in path",
            ),
            (
                "cargo run -- --arg1 --arg2",
                "cargo",
                "Double dash separator",
            ),
            ("npm run dev -- --port=3000", "npm", "NPM with args"),
        ];

        let mut results = String::new();
        for (cmd, proc, desc) in test_cases {
            let formatted = display.format_command(cmd, proc);
            results.push_str(&format!("{}: {}\n", desc, formatted));
        }

        insta::assert_snapshot!("command_special_chars", results);
    }

    #[test]
    fn snapshot_format_uptime_edge_cases() {
        let display = Display::new(false, false);
        let now = SystemTime::now();

        // Edge cases
        let zero_sec = now;
        let one_sec = now - std::time::Duration::from_secs(1);
        let fifty_nine_sec = now - std::time::Duration::from_secs(59);
        let one_hour = now - std::time::Duration::from_secs(3600);
        let twenty_three_hours = now - std::time::Duration::from_secs(82800);

        let results = format!(
            "0 seconds: {}\n1 second: {}\n59 seconds: {}\n1 hour: {}\n23 hours: {}",
            display.format_uptime(Some(zero_sec)),
            display.format_uptime(Some(one_sec)),
            display.format_uptime(Some(fifty_nine_sec)),
            display.format_uptime(Some(one_hour)),
            display.format_uptime(Some(twenty_three_hours))
        );

        insta::assert_snapshot!("uptime_edge_cases", results);
    }
}
