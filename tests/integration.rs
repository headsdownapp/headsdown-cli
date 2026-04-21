use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn help_flag_works() {
    Command::cargo_bin("hd")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("HeadsDown"));
}

#[test]
fn version_flag_works() {
    Command::cargo_bin("hd")
        .unwrap()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("hd"));
}

#[test]
fn unknown_command_fails() {
    Command::cargo_bin("hd")
        .unwrap()
        .arg("nonexistent")
        .assert()
        .failure();
}

#[test]
fn json_flag_is_recognized_on_status() {
    // status --json will fail on auth, but should NOT fail on arg parsing.
    // The error should be about authentication, not about unrecognized flags.
    Command::cargo_bin("hd")
        .unwrap()
        .args(["status", "--json"])
        .assert()
        .failure()
        .stderr(
            predicate::str::contains("Not authenticated")
                .or(predicate::str::contains("Could not determine config")),
        );
}

#[test]
fn subcommand_help_works() {
    for cmd in &[
        "auth",
        "status",
        "availability",
        "windows",
        "digest",
        "autoresponder",
        "verdict-settings",
        "proposals",
        "interrupt",
        "whoami",
        "busy",
        "online",
        "offline",
        "limited",
        "verdict",
        "presets",
        "grants",
        "override",
        "preset",
        "watch",
        "doctor",
        "update",
        "hook",
        "telemetry",
        "calibration",
        "outcome",
        "alias",
        "completions",
    ] {
        Command::cargo_bin("hd")
            .unwrap()
            .args([cmd, "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Usage:"));
    }
}

#[test]
fn windows_subcommand_help_works() {
    for cmd in &[
        ["windows", "list"],
        ["windows", "create"],
        ["windows", "update"],
        ["windows", "delete"],
    ] {
        Command::cargo_bin("hd")
            .unwrap()
            .args([cmd[0], cmd[1], "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Usage:"));
    }
}

#[test]
fn presets_subcommand_help_works() {
    Command::cargo_bin("hd")
        .unwrap()
        .args(["presets", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Usage:"));
}

#[test]
fn grants_subcommand_help_works() {
    for cmd in &[
        ["grants", "list-active"],
        ["grants", "list"],
        ["grants", "create"],
        ["grants", "revoke"],
        ["grants", "revoke-many"],
    ] {
        Command::cargo_bin("hd")
            .unwrap()
            .args([cmd[0], cmd[1], "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Usage:"));
    }
}

#[test]
fn override_subcommand_help_works() {
    for cmd in &[
        ["override", "get"],
        ["override", "set"],
        ["override", "clear"],
    ] {
        Command::cargo_bin("hd")
            .unwrap()
            .args([cmd[0], cmd[1], "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Usage:"));
    }
}

#[test]
fn digest_subcommand_help_works() {
    for cmd in &[
        ["digest", "list"],
        ["digest", "dismiss"],
        ["autoresponder", "get"],
        ["autoresponder", "set"],
        ["verdict-settings", "get"],
        ["verdict-settings", "set"],
    ] {
        Command::cargo_bin("hd")
            .unwrap()
            .args([cmd[0], cmd[1], "--help"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Usage:"));
    }
}

#[test]
fn windows_update_without_fields_fails_with_helpful_message() {
    Command::cargo_bin("hd")
        .unwrap()
        .args(["windows", "update", "window_123"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("No updates provided"));
}

#[test]
fn windows_create_missing_required_args_fails_at_parse() {
    Command::cargo_bin("hd")
        .unwrap()
        .args([
            "windows", "create", "--label", "Focus", "--mode", "busy", "--days", "Mon-Fri",
            "--start", "09:00:00",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "required arguments were not provided",
        ));
}

#[test]
fn completions_generates_output() {
    for shell in &["bash", "zsh", "fish"] {
        Command::cargo_bin("hd")
            .unwrap()
            .args(["completions", shell])
            .assert()
            .success()
            .stdout(predicate::str::is_empty().not());
    }
}
