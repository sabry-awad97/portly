use std::process::Command;

#[test]
fn test_help_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "--help"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Modern") || stdout.contains("port"));
    assert!(stdout.contains("list") || stdout.contains("Commands"));
}

#[test]
fn test_list_json_output() {
    let output = Command::new("cargo")
        .args(["run", "--", "list", "--json"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    
    // Should be valid JSON (either array or empty)
    assert!(stdout.starts_with('[') || stdout.trim().is_empty());
}

#[test]
fn test_config_path_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "config", "path"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("portly") && stdout.contains("config.toml"));
}

#[test]
fn test_config_init_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "config", "init"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("config") || stdout.contains("Created") || stdout.contains("exists"));
}

#[test]
fn test_ps_json_output() {
    let output = Command::new("cargo")
        .args(["run", "--", "ps", "--json"])
        .output()
        .expect("Failed to execute command");

    // Command should succeed
    assert!(output.status.success());
    
    // Output should be present (either JSON array or error message)
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(!stdout.is_empty() || !output.stderr.is_empty());
}

#[test]
fn test_invalid_command() {
    let output = Command::new("cargo")
        .args(["run", "--", "invalid-command"])
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
}

#[test]
fn test_details_invalid_port() {
    let output = Command::new("cargo")
        .args(["run", "--", "details", "99999"])
        .output()
        .expect("Failed to execute command");

    // Should fail or show "not in use"
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stdout.contains("not in use") || stderr.contains("not in use") || !output.status.success());
}
