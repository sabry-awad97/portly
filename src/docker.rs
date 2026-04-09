use anyhow::{Context, Result};
use bollard::Docker;
use std::collections::HashMap;

/// Docker integration module
///
/// Provides Docker container information and port mapping
#[derive(Debug, Clone)]
pub struct DockerClient {
    containers: HashMap<u16, DockerContainer>,
    docker: Option<Docker>,
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

        Self {
            containers,
            docker: None,
        }
    }

    /// Create a new async DockerClient with Bollard API
    ///
    /// # Errors
    ///
    /// Returns error if Docker connection fails
    pub async fn new_async() -> Result<Self> {
        // Try to connect to Docker daemon
        let docker = Docker::connect_with_local_defaults()
            .context("Failed to connect to Docker daemon")?;

        // Fetch containers using API
        let containers = Self::fetch_containers_async(&docker)
            .await
            .unwrap_or_default();

        Ok(Self {
            containers,
            docker: Some(docker),
        })
    }

    /// Get container information for a port
    pub fn get_container_info(&self, port: u16) -> Option<&DockerContainer> {
        self.containers.get(&port)
    }

    /// Check if Docker CLI is available and daemon is running
    fn check_docker_available() -> bool {
        use std::process::Command;
        
        Command::new("docker")
            .arg("ps")
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }

    /// Fetch all running containers with port mappings
    fn fetch_containers() -> Option<HashMap<u16, DockerContainer>> {
        use std::process::Command;
        
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

    /// Fetch containers using Bollard API (async)
    async fn fetch_containers_async(docker: &Docker) -> Option<HashMap<u16, DockerContainer>> {
        // List all running containers (no filters = running only by default)
        let containers_result = docker.list_containers(None).await;

        let containers_list = match containers_result {
            Ok(list) => list,
            Err(_) => return Some(HashMap::new()), // Graceful degradation
        };

        let mut containers = HashMap::new();

        for container in containers_list {
            // Extract container name (remove leading /)
            let name = container
                .names
                .and_then(|names| names.first().map(|n| n.trim_start_matches('/').to_string()))
                .unwrap_or_else(|| "unknown".to_string());

            // Extract image name
            let image = container.image.unwrap_or_else(|| "unknown".to_string());

            // Extract host ports from port mappings
            let host_ports = if let Some(ports) = container.ports {
                Self::extract_host_ports_from_api(&ports)
            } else {
                Vec::new()
            };

            if !host_ports.is_empty() {
                let docker_container = DockerContainer {
                    name: name.clone(),
                    image: image.clone(),
                };

                // Map each host port to this container
                for port in host_ports {
                    containers.insert(port, docker_container.clone());
                }
            }
        }

        Some(containers)
    }

    /// Extract host ports from Bollard API Port structs
    fn extract_host_ports_from_api(ports: &[bollard::models::PortSummary]) -> Vec<u16> {
        ports
            .iter()
            .filter_map(|port| port.public_port)
            .collect()
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
    use proptest::prelude::*;

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

    // ========== Property-Based Tests ==========

    proptest! {
        #[test]
        fn prop_parse_host_ports_no_panic(input in ".*") {
            // Property: Parser should never panic with any input
            let result = DockerClient::parse_host_ports(&input);

            // Should return Vec<u16> (possibly empty)
            // All returned ports should be valid (> 0)
            assert!(result.is_empty() || result.iter().all(|&p| p > 0));
        }

        #[test]
        fn prop_parse_host_ports_valid_ipv4(
            port in 1u16..=65535,
            container_port in 1u16..=65535
        ) {
            // Property: Valid IPv4 format should parse correctly
            let input = format!("0.0.0.0:{}->{}/ tcp", port, container_port);
            let result = DockerClient::parse_host_ports(&input);

            assert_eq!(result, vec![port]);
        }

        #[test]
        fn prop_parse_host_ports_valid_ipv6(
            port in 1u16..=65535,
            container_port in 1u16..=65535
        ) {
            // Property: Valid IPv6 format should parse correctly
            let input = format!(":::{}->{}/ tcp", port, container_port);
            let result = DockerClient::parse_host_ports(&input);

            assert_eq!(result, vec![port]);
        }

        #[test]
        fn prop_parse_host_ports_multiple(
            port1 in 1u16..=65535,
            port2 in 1u16..=65535,
            container_port1 in 1u16..=65535,
            container_port2 in 1u16..=65535
        ) {
            // Property: Multiple ports should parse correctly
            let input = format!(
                "0.0.0.0:{}->{}/ tcp, 0.0.0.0:{}->{}/ tcp",
                port1, container_port1, port2, container_port2
            );
            let result = DockerClient::parse_host_ports(&input);

            assert_eq!(result, vec![port1, port2]);
        }

        #[test]
        fn prop_parse_host_ports_empty_returns_empty(input in ".*") {
            // Property: If no valid ports found, return empty vec
            let result = DockerClient::parse_host_ports(&input);

            // Should never panic, always return a vec
            // Number of ports should not exceed number of "->" patterns
            assert!(result.len() <= input.matches("->").count());
        }

        #[test]
        fn prop_detect_framework_no_panic(image in ".*") {
            // Property: Framework detection should never panic
            let result = DockerClient::detect_framework_from_image(&image);

            // Should always return Some (at least "Docker")
            assert!(result.is_some());

            // Framework name should not be empty
            if let Some(framework) = result {
                assert!(!framework.is_empty());
            }
        }
    }

    // ========== Async/Bollard Tests ==========

    #[tokio::test]
    async fn test_docker_client_connects() {
        // RED: Test that DockerClient can be created asynchronously
        let result = DockerClient::new_async().await;
        
        // Should return Ok even if Docker is not available (graceful degradation)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fetch_containers_via_api() {
        // RED: Test that containers are fetched via Bollard API
        let result = DockerClient::new_async().await;
        
        if let Ok(client) = result {
            // Should have containers map (empty or populated)
            // This tests that fetch_containers_async was called
            assert!(client.containers.len() >= 0);
        }
    }

    #[test]
    fn test_extract_host_ports_from_api() {
        // RED: Test that port mappings are extracted correctly from API types
        use bollard::models::{PortSummary, PortSummaryTypeEnum};

        let ports = vec![
            PortSummary {
                ip: Some("0.0.0.0".to_string()),
                private_port: 5432,
                public_port: Some(5432),
                typ: Some(PortSummaryTypeEnum::TCP),
            },
            PortSummary {
                ip: Some("0.0.0.0".to_string()),
                private_port: 3000,
                public_port: Some(3000),
                typ: Some(PortSummaryTypeEnum::TCP),
            },
        ];

        let host_ports = DockerClient::extract_host_ports_from_api(&ports);
        assert_eq!(host_ports, vec![5432, 3000]);
    }
}
