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
        "whoami",
        "busy",
        "online",
        "offline",
        "limited",
        "verdict",
        "presets",
        "preset",
        "watch",
        "doctor",
        "update",
        "hook",
        "telemetry",
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
