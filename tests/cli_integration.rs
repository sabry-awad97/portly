use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_help_command() {
    Command::cargo_bin("portly")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Portly"))
        .stdout(predicate::str::contains("Commands"));
}

#[test]
fn test_version_command() {
    Command::cargo_bin("portly")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("portly"));
}

#[test]
fn test_list_command() {
    Command::cargo_bin("portly")
        .unwrap()
        .arg("list")
        .assert()
        .success();
}

#[test]
fn test_list_json_output() {
    Command::cargo_bin("portly")
        .unwrap()
        .args(["list", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("["));
}

#[test]
fn test_list_with_all_flag() {
    Command::cargo_bin("portly")
        .unwrap()
        .args(["list", "--all"])
        .assert()
        .success();
}

#[test]
fn test_list_with_no_color() {
    Command::cargo_bin("portly")
        .unwrap()
        .args(["list", "--no-color"])
        .assert()
        .success();
}

#[test]
fn test_config_path_command() {
    Command::cargo_bin("portly")
        .unwrap()
        .args(["config", "path"])
        .assert()
        .success()
        .stdout(predicate::str::contains("portly"))
        .stdout(predicate::str::contains("config.toml"));
}

#[test]
fn test_config_init_command() {
    Command::cargo_bin("portly")
        .unwrap()
        .args(["config", "init"])
        .assert()
        .success()
        .stdout(
            predicate::str::contains("config")
                .or(predicate::str::contains("Created"))
                .or(predicate::str::contains("exists")),
        );
}

#[test]
fn test_config_reset_command() {
    Command::cargo_bin("portly")
        .unwrap()
        .args(["config", "reset"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Reset").or(predicate::str::contains("config")));
}

#[test]
fn test_ps_command() {
    Command::cargo_bin("portly")
        .unwrap()
        .arg("ps")
        .assert()
        .success();
}

#[test]
fn test_ps_json_output() {
    Command::cargo_bin("portly")
        .unwrap()
        .args(["ps", "--json"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("{"))
        .stdout(predicate::str::contains("\"processes\""));
}

#[test]
fn test_ps_with_all_flag() {
    Command::cargo_bin("portly")
        .unwrap()
        .args(["ps", "--all"])
        .assert()
        .success();
}

#[test]
fn test_invalid_command() {
    Command::cargo_bin("portly")
        .unwrap()
        .arg("invalid-command")
        .assert()
        .failure()
        .stderr(predicate::str::contains("unrecognized").or(predicate::str::contains("invalid")));
}

#[test]
fn test_details_invalid_port() {
    Command::cargo_bin("portly")
        .unwrap()
        .args(["details", "99999"])
        .assert()
        .failure() // Port out of range returns error code 2
        .stderr(predicate::str::contains("not in 0..=65535"));
}

#[test]
fn test_kill_without_target() {
    Command::cargo_bin("portly")
        .unwrap()
        .arg("kill")
        .assert()
        .failure()
        .stderr(predicate::str::contains("required").or(predicate::str::contains("argument")));
}

#[test]
fn test_clean_dry_run() {
    Command::cargo_bin("portly")
        .unwrap()
        .arg("clean")
        .assert()
        .success();
}

#[test]
fn test_watch_help() {
    Command::cargo_bin("portly")
        .unwrap()
        .args(["watch", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("watch").or(predicate::str::contains("interval")));
}

#[test]
fn test_global_json_flag() {
    Command::cargo_bin("portly")
        .unwrap()
        .args(["--json", "list"])
        .assert()
        .success()
        .stdout(predicate::str::starts_with("["));
}

#[test]
fn test_global_no_color_flag() {
    Command::cargo_bin("portly")
        .unwrap()
        .args(["--no-color", "list"])
        .assert()
        .success();
}

#[test]
fn test_config_subcommand_help() {
    Command::cargo_bin("portly")
        .unwrap()
        .args(["config", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("path"))
        .stdout(predicate::str::contains("reset"));
}

#[test]
fn test_details_with_json() {
    // Test with a port that likely doesn't exist
    Command::cargo_bin("portly")
        .unwrap()
        .args(["details", "65000", "--json"])
        .assert()
        .code(predicate::in_iter([0, 1])); // May succeed or fail depending on port availability
}

#[test]
fn test_kill_with_force_flag() {
    // Test that the force flag is accepted (won't actually kill anything without a valid target)
    Command::cargo_bin("portly")
        .unwrap()
        .args(["kill", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("force").or(predicate::str::contains("-f")));
}

#[test]
fn test_clean_with_execute_flag() {
    Command::cargo_bin("portly")
        .unwrap()
        .args(["clean", "--execute"])
        .assert()
        .success();
}
