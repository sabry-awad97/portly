use std::collections::HashMap;
use std::process::Command;

/// Docker integration module
///
/// Provides Docker container information and port mapping
#[derive(Debug, Clone)]
pub struct DockerClient {
    containers: HashMap<u16, DockerContainer>,
}

/// Docker container information
#[derive(Debug, Clone)]
pub struct DockerContainer {
    pub name: String,
    pub image: String,
}

impl DockerClient {
    pub fn new() -> Self {
        let available = Self::check_docker_available();
        let containers = if available {
            Self::fetch_containers().unwrap_or_default()
        } else {
            HashMap::new()
        };

        Self { containers }
    }

    /// Get container information for a port
    pub fn get_container_info(&self, port: u16) -> Option<&DockerContainer> {
        self.containers.get(&port)
    }

    /// Check if Docker CLI is available and daemon is running
    fn check_docker_available() -> bool {
        Command::new("docker")
            .arg("ps")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Fetch all running containers with port mappings
    fn fetch_containers() -> Option<HashMap<u16, DockerContainer>> {
        let output = Command::new("docker")
            .args([
                "ps",
                "--format",
                "{{.Ports}}\t{{.Names}}\t{{.Image}}\t{{.Status}}",
            ])
            .output()
            .ok()?;

        if !output.status.success() {
            return None;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        let mut containers = HashMap::new();

        for line in stdout.lines() {
            let parts: Vec<&str> = line.split('\t').collect();
            if parts.len() < 4 {
                continue;
            }

            let ports_str = parts[0];
            let name = parts[1].to_string();
            let image = parts[2].to_string();

            // Parse host ports from port mapping
            let host_ports = Self::parse_host_ports(ports_str);

            if !host_ports.is_empty() {
                let container = DockerContainer {
                    name: name.clone(),
                    image: image.clone(),
                };

                // Map each host port to this container
                for port in host_ports {
                    containers.insert(port, container.clone());
                }
            }
        }

        Some(containers)
    }

    /// Parse host ports from Docker port mapping string
    /// Examples:
    /// - "0.0.0.0:5432->5432/tcp" → \[5432\]
    /// - "0.0.0.0:3000->3000/tcp, 0.0.0.0:3001->3001/tcp" → \[3000, 3001\]
    /// - ":::5432->5432/tcp" → \[5432\]
    fn parse_host_ports(ports_str: &str) -> Vec<u16> {
        let mut ports = Vec::new();

        // Regex pattern: (?:0\.0\.0\.0|:::):(\d+)->
        // Matches: 0.0.0.0:5432-> or :::5432->
        for part in ports_str.split(',') {
            let part = part.trim();

            // Find the host port (before ->)
            if let Some(arrow_pos) = part.find("->") {
                let before_arrow = &part[..arrow_pos];

                // Find the last colon before ->
                if let Some(colon_pos) = before_arrow.rfind(':') {
                    let port_str = &before_arrow[colon_pos + 1..];
                    if let Ok(port) = port_str.parse::<u16>() {
                        ports.push(port);
                    }
                }
            }
        }

        ports
    }

    /// Detect framework from Docker image name
    pub fn detect_framework_from_image(image: &str) -> Option<String> {
        let img = image.to_lowercase();

        if img.contains("postgres") {
            Some("PostgreSQL".to_string())
        } else if img.contains("redis") {
            Some("Redis".to_string())
        } else if img.contains("nginx") {
            Some("nginx".to_string())
        } else if img.contains("mongo") {
            Some("MongoDB".to_string())
        } else if img.contains("mysql") || img.contains("mariadb") {
            Some("MySQL".to_string())
        } else if img.contains("rabbitmq") {
            Some("RabbitMQ".to_string())
        } else if img.contains("localstack") {
            Some("LocalStack".to_string())
        } else if img.contains("elasticsearch") {
            Some("Elasticsearch".to_string())
        } else {
            Some("Docker".to_string())
        }
    }

    /// Parse Docker status to human-readable uptime
    /// "Up 10 days" → "10d"
    /// "Up 2 hours" → "2h"
    /// "Up 30 minutes" → "30m"
    pub fn _parse_docker_uptime(status: &str) -> Option<String> {
        let status_lower = status.to_lowercase();

        if !status_lower.starts_with("up ") {
            return None;
        }

        // Extract the time part after "Up "
        let time_part = &status[3..];

        // Parse different time formats
        if time_part.contains("day") {
            // "10 days" or "1 day"
            if let Some(num_str) = time_part.split_whitespace().next()
                && let Ok(days) = num_str.parse::<u32>()
            {
                return Some(format!("{}d", days));
            }
        } else if time_part.contains("hour") {
            // "2 hours" or "1 hour"
            if let Some(num_str) = time_part.split_whitespace().next()
                && let Ok(hours) = num_str.parse::<u32>()
            {
                return Some(format!("{}h", hours));
            }
        } else if time_part.contains("minute") {
            // "30 minutes" or "1 minute"
            if let Some(num_str) = time_part.split_whitespace().next()
                && let Ok(minutes) = num_str.parse::<u32>()
            {
                return Some(format!("{}m", minutes));
            }
        } else if time_part.contains("second") {
            // "45 seconds" or "1 second"
            return Some("0m".to_string());
        }

        None
    }
}

impl Default for DockerClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_host_ports_single() {
        let ports = DockerClient::parse_host_ports("0.0.0.0:5432->5432/tcp");
        assert_eq!(ports, vec![5432]);
    }

    #[test]
    fn test_parse_host_ports_multiple() {
        let ports =
            DockerClient::parse_host_ports("0.0.0.0:3000->3000/tcp, 0.0.0.0:3001->3001/tcp");
        assert_eq!(ports, vec![3000, 3001]);
    }

    #[test]
    fn test_parse_host_ports_ipv6() {
        let ports = DockerClient::parse_host_ports(":::5432->5432/tcp");
        assert_eq!(ports, vec![5432]);
    }

    #[test]
    fn test_parse_host_ports_empty() {
        let ports = DockerClient::parse_host_ports("");
        assert!(ports.is_empty());
    }

    #[test]
    fn test_detect_framework_postgres() {
        assert_eq!(
            DockerClient::detect_framework_from_image("postgres:14"),
            Some("PostgreSQL".to_string())
        );
    }

    #[test]
    fn test_detect_framework_redis() {
        assert_eq!(
            DockerClient::detect_framework_from_image("redis:7"),
            Some("Redis".to_string())
        );
    }

    #[test]
    fn test_detect_framework_nginx() {
        assert_eq!(
            DockerClient::detect_framework_from_image("nginx:latest"),
            Some("nginx".to_string())
        );
    }

    #[test]
    fn test_detect_framework_mongo() {
        assert_eq!(
            DockerClient::detect_framework_from_image("mongo:5"),
            Some("MongoDB".to_string())
        );
    }

    #[test]
    fn test_parse_docker_uptime_days() {
        assert_eq!(
            DockerClient::_parse_docker_uptime("Up 10 days"),
            Some("10d".to_string())
        );
    }

    #[test]
    fn test_parse_docker_uptime_hours() {
        assert_eq!(
            DockerClient::_parse_docker_uptime("Up 2 hours"),
            Some("2h".to_string())
        );
    }

    #[test]
    fn test_parse_docker_uptime_minutes() {
        assert_eq!(
            DockerClient::_parse_docker_uptime("Up 30 minutes"),
            Some("30m".to_string())
        );
    }

    #[test]
    fn test_parse_docker_uptime_seconds() {
        assert_eq!(
            DockerClient::_parse_docker_uptime("Up 45 seconds"),
            Some("0m".to_string())
        );
    }
}
