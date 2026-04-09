use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Framework detection module
///
/// Detects development frameworks from:
/// - Command line arguments
/// - Working directory markers (package.json, Cargo.toml, etc.)
/// - Config file parsing
/// - Docker image names
pub struct FrameworkDetector {
    cache: HashMap<String, Option<String>>,
}

impl FrameworkDetector {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Detect framework from command line and working directory
    pub fn detect(&mut self, command: &str, working_dir: Option<&str>) -> Option<String> {
        // Try command line detection first (fastest)
        if let Some(framework) = self.detect_from_command(command) {
            return Some(framework);
        }

        // Try working directory detection (slower)
        if let Some(dir) = working_dir {
            // Check cache first
            if let Some(cached) = self.cache.get(dir) {
                return cached.clone();
            }

            let framework = self.detect_from_directory(dir);
            self.cache.insert(dir.to_string(), framework.clone());
            return framework;
        }

        None
    }

    /// Detect framework from command line
    fn detect_from_command(&self, command: &str) -> Option<String> {
        let cmd_lower = command.to_lowercase();

        // Node.js frameworks
        if cmd_lower.contains("next") {
            return Some("Next.js".to_string());
        }
        if cmd_lower.contains("vite") {
            return Some("Vite".to_string());
        }
        if cmd_lower.contains("webpack") {
            return Some("Webpack".to_string());
        }
        if cmd_lower.contains("parcel") {
            return Some("Parcel".to_string());
        }
        if cmd_lower.contains("nuxt") {
            return Some("Nuxt".to_string());
        }
        if cmd_lower.contains("gatsby") {
            return Some("Gatsby".to_string());
        }

        // Python frameworks
        if cmd_lower.contains("django") || cmd_lower.contains("manage.py") {
            return Some("Django".to_string());
        }
        if cmd_lower.contains("flask") {
            return Some("Flask".to_string());
        }
        if cmd_lower.contains("fastapi") || cmd_lower.contains("uvicorn") {
            return Some("FastAPI".to_string());
        }

        // Rust
        if cmd_lower.contains("cargo run") || cmd_lower.contains("cargo-watch") {
            return Some("Rust".to_string());
        }
        if cmd_lower.contains("trunk") {
            return Some("Trunk".to_string());
        }

        // Go
        if cmd_lower.contains("go run") || cmd_lower.contains("air") {
            return Some("Go".to_string());
        }

        // Ruby
        if cmd_lower.contains("rails") {
            return Some("Rails".to_string());
        }
        if cmd_lower.contains("puma") || cmd_lower.contains("unicorn") {
            return Some("Ruby".to_string());
        }

        // PHP
        if cmd_lower.contains("laravel") || cmd_lower.contains("artisan") {
            return Some("Laravel".to_string());
        }
        if cmd_lower.contains("symfony") {
            return Some("Symfony".to_string());
        }

        // .NET
        if cmd_lower.contains("dotnet") {
            return Some(".NET".to_string());
        }

        // Java
        if cmd_lower.contains("spring") {
            return Some("Spring".to_string());
        }

        // Docker
        if cmd_lower.contains("docker") {
            return self.detect_docker_framework(command);
        }

        None
    }

    /// Detect framework from Docker image name
    fn detect_docker_framework(&self, command: &str) -> Option<String> {
        let cmd_lower = command.to_lowercase();

        if cmd_lower.contains("postgres") {
            return Some("PostgreSQL".to_string());
        }
        if cmd_lower.contains("redis") {
            return Some("Redis".to_string());
        }
        if cmd_lower.contains("nginx") {
            return Some("nginx".to_string());
        }
        if cmd_lower.contains("mongo") {
            return Some("MongoDB".to_string());
        }
        if cmd_lower.contains("mysql") {
            return Some("MySQL".to_string());
        }
        if cmd_lower.contains("rabbitmq") {
            return Some("RabbitMQ".to_string());
        }

        Some("Docker".to_string())
    }

    /// Detect framework from working directory
    fn detect_from_directory(&self, path: &str) -> Option<String> {
        let dir = Path::new(path);

        // Find project root
        let project_root = self.find_project_root(dir)?;

        // Check for package.json (Node.js)
        if let Some(framework) = self.detect_from_package_json(&project_root) {
            return Some(framework);
        }

        // Check for Cargo.toml (Rust)
        if project_root.join("Cargo.toml").exists() {
            return Some("Rust".to_string());
        }

        // Check for go.mod (Go)
        if project_root.join("go.mod").exists() {
            return Some("Go".to_string());
        }

        // Check for manage.py (Django)
        if project_root.join("manage.py").exists() {
            return Some("Django".to_string());
        }

        // Check for pyproject.toml (Python)
        if project_root.join("pyproject.toml").exists() {
            return Some("Python".to_string());
        }

        // Check for Gemfile (Ruby)
        if project_root.join("Gemfile").exists() {
            return Some("Ruby".to_string());
        }

        // Check for composer.json (PHP)
        if project_root.join("composer.json").exists() {
            return Some("PHP".to_string());
        }

        None
    }

    /// Find project root by walking up directory tree
    fn find_project_root(&self, start_dir: &Path) -> Option<PathBuf> {
        let mut current = start_dir;
        let mut depth = 0;

        while depth < 10 {
            // Check for project markers
            if current.join("package.json").exists()
                || current.join("Cargo.toml").exists()
                || current.join("go.mod").exists()
                || current.join("pyproject.toml").exists()
                || current.join("Gemfile").exists()
                || current.join("composer.json").exists()
            {
                return Some(current.to_path_buf());
            }

            // Move up one directory
            if let Some(parent) = current.parent() {
                current = parent;
                depth += 1;
            } else {
                break;
            }
        }

        None
    }

    /// Detect framework from package.json
    fn detect_from_package_json(&self, project_root: &Path) -> Option<String> {
        let package_json_path = project_root.join("package.json");
        if !package_json_path.exists() {
            return None;
        }

        // Read and parse package.json
        let content = fs::read_to_string(&package_json_path).ok()?;
        let json: serde_json::Value = serde_json::from_str(&content).ok()?;

        // Check dependencies
        let deps = json.get("dependencies")?;
        let dev_deps = json.get("devDependencies");

        // Check for frameworks in dependencies
        if deps.get("next").is_some() {
            return Some("Next.js".to_string());
        }
        if deps.get("vite").is_some() {
            return Some("Vite".to_string());
        }
        if deps.get("@angular/core").is_some() {
            return Some("Angular".to_string());
        }
        if deps.get("vue").is_some() {
            return Some("Vue".to_string());
        }
        if deps.get("react").is_some() {
            return Some("React".to_string());
        }
        if deps.get("express").is_some() {
            return Some("Express".to_string());
        }
        if deps.get("nuxt").is_some() {
            return Some("Nuxt".to_string());
        }
        if deps.get("gatsby").is_some() {
            return Some("Gatsby".to_string());
        }

        // Check devDependencies
        if let Some(dev_deps) = dev_deps {
            if dev_deps.get("vite").is_some() {
                return Some("Vite".to_string());
            }
            if dev_deps.get("@angular/cli").is_some() {
                return Some("Angular".to_string());
            }
        }

        // Default to Node.js if no specific framework found
        Some("Node.js".to_string())
    }
}

impl Default for FrameworkDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn test_detect_from_command_nextjs() {
        let detector = FrameworkDetector::new();
        assert_eq!(
            detector.detect_from_command("node /path/to/next dev"),
            Some("Next.js".to_string())
        );
    }

    #[test]
    fn test_detect_from_command_vite() {
        let detector = FrameworkDetector::new();
        assert_eq!(
            detector.detect_from_command("node vite --port 3000"),
            Some("Vite".to_string())
        );
    }

    #[test]
    fn test_detect_from_command_django() {
        let detector = FrameworkDetector::new();
        assert_eq!(
            detector.detect_from_command("python manage.py runserver"),
            Some("Django".to_string())
        );
    }

    #[test]
    fn test_detect_from_command_rust() {
        let detector = FrameworkDetector::new();
        assert_eq!(
            detector.detect_from_command("cargo run --bin server"),
            Some("Rust".to_string())
        );
    }

    #[test]
    fn test_detect_from_command_go() {
        let detector = FrameworkDetector::new();
        assert_eq!(
            detector.detect_from_command("go run main.go"),
            Some("Go".to_string())
        );
    }

    #[test]
    fn test_detect_docker_postgres() {
        let detector = FrameworkDetector::new();
        assert_eq!(
            detector.detect_docker_framework("docker run postgres:14"),
            Some("PostgreSQL".to_string())
        );
    }

    #[test]
    fn test_detect_docker_redis() {
        let detector = FrameworkDetector::new();
        assert_eq!(
            detector.detect_docker_framework("docker run redis:latest"),
            Some("Redis".to_string())
        );
    }

    #[test]
    fn test_detect_from_command_none() {
        let detector = FrameworkDetector::new();
        assert_eq!(detector.detect_from_command("unknown command"), None);
    }

    // ========== Property-Based Tests ==========

    proptest! {
        #[test]
        fn prop_framework_detection_no_panic(command in ".*") {
            // Property: Detection should never panic
            let mut detector = FrameworkDetector::new();
            let result = detector.detect(&command, None);

            // Should return Option<String> (possibly None)
            if let Some(framework) = result {
                assert!(!framework.is_empty());
            }
        }

        #[test]
        fn prop_framework_detection_consistency(command in ".*") {
            // Property: Same command should return same result
            let mut detector1 = FrameworkDetector::new();
            let mut detector2 = FrameworkDetector::new();

            let result1 = detector1.detect(&command, None);
            let result2 = detector2.detect(&command, None);

            assert_eq!(result1, result2);
        }

        #[test]
        fn prop_framework_detection_with_working_dir(
            command in ".*",
            working_dir in ".*"
        ) {
            // Property: Detection with working dir should never panic
            let mut detector = FrameworkDetector::new();
            let result = detector.detect(&command, Some(&working_dir));

            // Should return Option<String>
            if let Some(framework) = result {
                assert!(!framework.is_empty());
            }
        }

        #[test]
        fn prop_detect_from_command_no_panic(command in ".*") {
            // Property: Command detection should never panic
            let detector = FrameworkDetector::new();
            let result = detector.detect_from_command(&command);

            // Should return Option<String>
            if let Some(framework) = result {
                assert!(!framework.is_empty());
            }
        }

        #[test]
        fn prop_detect_docker_framework_no_panic(command in ".*") {
            // Property: Docker framework detection should never panic
            let detector = FrameworkDetector::new();
            let result = detector.detect_docker_framework(&command);

            // Should return Option<String>
            if let Some(framework) = result {
                assert!(!framework.is_empty());
            }
        }

        #[test]
        fn prop_cache_consistency(
            command in ".*",
            working_dir in "[a-zA-Z0-9_/\\\\.-]{1,50}"
        ) {
            // Property: Cache should return consistent results
            let mut detector = FrameworkDetector::new();

            // First call (populates cache)
            let result1 = detector.detect(&command, Some(&working_dir));

            // Second call (uses cache)
            let result2 = detector.detect(&command, Some(&working_dir));

            // Should be identical
            assert_eq!(result1, result2);
        }
    }
}
