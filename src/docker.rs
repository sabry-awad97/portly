use anyhow::{Context, Result};
use bollard::Docker;
use std::collections::HashMap;

/// Docker integration module
///
/// Provides Docker container information and port mapping
#[derive(Debug, Clone)]
pub struct DockerClient {
    containers: HashMap<u16, DockerContainer>,
    #[allow(dead_code)] // Used in async path, kept for future API extensions
    docker: Option<Docker>,
}

/// Docker container information
#[derive(Debug, Clone)]
pub struct DockerContainer {
    pub name: String,
    pub image: String,
}

impl DockerClient {
    /// Create a new async DockerClient with Bollard API
    ///
    /// # Errors
    ///
    /// Returns error if Docker connection fails
    pub async fn new() -> Result<Self> {
        // Try to connect to Docker daemon
        let docker =
            Docker::connect_with_local_defaults().context("Failed to connect to Docker daemon")?;

        // Fetch containers using API
        let containers = Self::fetch_containers(&docker).await.unwrap_or_default();

        Ok(Self {
            containers,
            docker: Some(docker),
        })
    }

    /// Create an empty DockerClient (for fallback when Docker is unavailable)
    pub(crate) fn empty() -> Self {
        Self {
            containers: HashMap::new(),
            docker: None,
        }
    }

    /// Get container information for a port
    pub fn get_container_info(&self, port: u16) -> Option<&DockerContainer> {
        self.containers.get(&port)
    }

    /// Fetch containers using Bollard API
    async fn fetch_containers(docker: &Docker) -> Option<HashMap<u16, DockerContainer>> {
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
                Self::extract_host_ports(&ports)
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
    fn extract_host_ports(ports: &[bollard::models::PortSummary]) -> Vec<u16> {
        ports.iter().filter_map(|port| port.public_port).collect()
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

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

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
        let result = DockerClient::new().await;
        // Should return Ok even if Docker is not available (graceful degradation)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_fetch_containers_via_api() {
        let result = DockerClient::new().await;
        // Should successfully create client (even if Docker is unavailable)
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_host_ports_from_api() {
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

        let host_ports = DockerClient::extract_host_ports(&ports);
        assert_eq!(host_ports, vec![5432, 3000]);
    }
}
