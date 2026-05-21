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
        "install",
        "doctor",
        "update",
        "remove",
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
fn install_claude_dry_run_does_not_write() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".claude")).unwrap();

    Command::cargo_bin("hd")
        .unwrap()
        .env("HOME", dir.path())
        .args(["install", "claude", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Would install the HeadsDown integration for Claude Code",
        ));

    assert!(!dir
        .path()
        .join(".claude/commands/headsdown/referee.md")
        .exists());
}

#[test]
fn install_claude_preserves_user_owned_command() {
    let dir = tempfile::tempdir().unwrap();
    let command_dir = dir.path().join(".claude/commands/headsdown");
    std::fs::create_dir_all(&command_dir).unwrap();
    let command_path = command_dir.join("referee.md");
    std::fs::write(&command_path, "user-owned command").unwrap();

    Command::cargo_bin("hd")
        .unwrap()
        .env("HOME", dir.path())
        .args(["install", "claude"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "user-owned integration artifact was preserved",
        ));

    assert_eq!(
        std::fs::read_to_string(command_path).unwrap(),
        "user-owned command"
    );
}

#[test]
fn install_claude_is_idempotent() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".claude")).unwrap();

    Command::cargo_bin("hd")
        .unwrap()
        .env("HOME", dir.path())
        .args(["install", "claude"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Installed the HeadsDown integration for Claude Code",
        ));

    Command::cargo_bin("hd")
        .unwrap()
        .env("HOME", dir.path())
        .args(["install", "claude"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "HeadsDown integration is already current",
        ));

    let command =
        std::fs::read_to_string(dir.path().join(".claude/commands/headsdown/referee.md")).unwrap();
    assert!(command.contains("headsdown-cli managed"));
    assert!(!command.contains("<<'HEADSDOWN_REFEREE_EVIDENCE'"));
    assert!(!command.contains("printf '%s' \"$ARGUMENTS\""));
}

#[test]
fn install_all_dry_run_shows_detected_tools() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".claude")).unwrap();
    std::fs::create_dir_all(dir.path().join(".pi/agent")).unwrap();
    std::fs::create_dir_all(dir.path().join(".codex")).unwrap();

    Command::cargo_bin("hd")
        .unwrap()
        .env("HOME", dir.path())
        .args(["install", "--all", "--dry-run"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Detected supported tools"))
        .stdout(predicate::str::contains("Claude Code"))
        .stdout(predicate::str::contains("Pi"))
        .stdout(predicate::str::contains("Codex"));
}

#[test]
fn install_all_json_outputs_valid_json_without_human_preamble() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".claude")).unwrap();

    let assert = Command::cargo_bin("hd")
        .unwrap()
        .env("HOME", dir.path())
        .args(["install", "--all", "--dry-run", "--json"])
        .assert()
        .success();

    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value[0]["tool"], "claude");
    assert_eq!(value[0]["status"], "planned");
}

#[test]
fn bulk_install_json_requires_yes_without_prompting() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".claude")).unwrap();

    let assert = Command::cargo_bin("hd")
        .unwrap()
        .env("HOME", dir.path())
        .args(["install", "--all", "--json"])
        .assert()
        .success();

    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value[0]["status"], "skipped");
    assert!(!output.contains("[y/N]"));
}

#[test]
fn bulk_install_prompts_and_does_not_write_on_no() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".claude")).unwrap();

    Command::cargo_bin("hd")
        .unwrap()
        .env("HOME", dir.path())
        .args(["install", "--all"])
        .write_stdin("n\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("No changes made"));

    assert!(!dir
        .path()
        .join(".claude/commands/headsdown/referee.md")
        .exists());
}

#[test]
fn bulk_update_prompts_and_does_not_write_on_no() {
    let dir = tempfile::tempdir().unwrap();
    let command_dir = dir.path().join(".claude/commands/headsdown");
    std::fs::create_dir_all(&command_dir).unwrap();
    let command_path = command_dir.join("referee.md");
    std::fs::write(
        &command_path,
        "<!-- headsdown-cli managed: claude-referee-command v1 -->\nstale",
    )
    .unwrap();

    Command::cargo_bin("hd")
        .unwrap()
        .env("HOME", dir.path())
        .args(["update", "--all"])
        .write_stdin("n\n")
        .assert()
        .success()
        .stdout(predicate::str::contains("No changes made"));

    assert_eq!(
        std::fs::read_to_string(command_path).unwrap(),
        "<!-- headsdown-cli managed: claude-referee-command v1 -->\nstale"
    );
}

#[test]
fn conflicting_tool_and_all_flags_fail() {
    Command::cargo_bin("hd")
        .unwrap()
        .args(["install", "claude", "--all"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Pass either a tool or --all"));

    Command::cargo_bin("hd")
        .unwrap()
        .args(["update", "claude", "--all"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Pass either a tool or --all"));
}

#[test]
fn update_cli_rejects_integration_flags() {
    Command::cargo_bin("hd")
        .unwrap()
        .args(["update", "claude", "--cli", "--dry-run"])
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "--cli cannot be combined with integration update options",
        ));
}

#[test]
fn update_claude_repairs_stale_managed_command() {
    let dir = tempfile::tempdir().unwrap();
    let command_dir = dir.path().join(".claude/commands/headsdown");
    std::fs::create_dir_all(&command_dir).unwrap();
    let command_path = command_dir.join("referee.md");
    std::fs::write(
        &command_path,
        "<!-- headsdown-cli managed: claude-referee-command v1 -->\nstale",
    )
    .unwrap();

    Command::cargo_bin("hd")
        .unwrap()
        .env("HOME", dir.path())
        .args(["update", "claude"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Updated the HeadsDown integration for Claude Code",
        ));

    let command = std::fs::read_to_string(command_path).unwrap();
    assert!(command.contains("Run `headsdown-claude referee`"));
    assert!(!command.contains("stale"));
}

#[test]
fn remove_pi_preserves_user_packages() {
    let dir = tempfile::tempdir().unwrap();
    let agent_dir = dir.path().join(".pi/agent");
    std::fs::create_dir_all(&agent_dir).unwrap();
    std::fs::write(
        agent_dir.join("settings.json"),
        r#"{"packages":["existing","git:github.com/headsdownapp/headsdown-pi"]}"#,
    )
    .unwrap();

    Command::cargo_bin("hd")
        .unwrap()
        .env("HOME", dir.path())
        .args(["remove", "pi"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Removed the HeadsDown integration for Pi",
        ));

    let raw = std::fs::read_to_string(agent_dir.join("settings.json")).unwrap();
    assert!(raw.contains("existing"));
    assert!(!raw.contains("headsdown-pi"));
}

#[test]
fn codex_install_preserves_unmanaged_headsdown_table() {
    let dir = tempfile::tempdir().unwrap();
    let codex_dir = dir.path().join(".codex");
    std::fs::create_dir_all(&codex_dir).unwrap();
    let config_path = codex_dir.join("config.toml");
    std::fs::write(
        &config_path,
        "[mcp_servers.headsdown]\ncommand = \"custom\"\n",
    )
    .unwrap();

    Command::cargo_bin("hd")
        .unwrap()
        .env("HOME", dir.path())
        .args(["install", "codex"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "user-owned integration artifact was preserved",
        ));

    assert_eq!(
        std::fs::read_to_string(config_path).unwrap(),
        "[mcp_servers.headsdown]\ncommand = \"custom\"\n"
    );
}

#[test]
fn codex_doctor_flags_unmanaged_duplicate_headsdown_table() {
    let dir = tempfile::tempdir().unwrap();
    let codex_dir = dir.path().join(".codex");
    std::fs::create_dir_all(&codex_dir).unwrap();
    std::fs::write(
        codex_dir.join("config.toml"),
        "[mcp_servers.headsdown]\ncommand = \"custom\"\n\n# <headsdown-cli managed: codex-mcp v1>\n[mcp_servers.headsdown]\ncommand = \"npx\"\nargs = [\"-y\", \"headsdown-claude\"]\n# </headsdown-cli managed: codex-mcp v1>\n",
    )
    .unwrap();

    let assert = Command::cargo_bin("hd")
        .unwrap()
        .env("HOME", dir.path())
        .args(["doctor", "codex", "--json"])
        .assert()
        .success();

    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value[0]["installed"], true);
    assert_eq!(value[0]["current"], false);
}

#[test]
fn remove_codex_preserves_user_config() {
    let dir = tempfile::tempdir().unwrap();
    let codex_dir = dir.path().join(".codex");
    std::fs::create_dir_all(&codex_dir).unwrap();
    std::fs::write(
        codex_dir.join("config.toml"),
        "keep = true\n\n# <headsdown-cli managed: codex-mcp v1>\n[mcp_servers.headsdown]\ncommand = \"npx\"\nargs = [\"-y\", \"headsdown-claude\"]\n# </headsdown-cli managed: codex-mcp v1>\n\nkeep_again = true\n",
    )
    .unwrap();

    Command::cargo_bin("hd")
        .unwrap()
        .env("HOME", dir.path())
        .args(["remove", "codex"])
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Removed the HeadsDown integration for Codex",
        ));

    let raw = std::fs::read_to_string(codex_dir.join("config.toml")).unwrap();
    assert!(raw.contains("keep = true"));
    assert!(raw.contains("keep_again = true"));
    assert!(!raw.contains("mcp_servers.headsdown"));
}

#[test]
fn doctor_all_json_reports_supported_tools_without_paths() {
    let dir = tempfile::tempdir().unwrap();
    let assert = Command::cargo_bin("hd")
        .unwrap()
        .env("HOME", dir.path())
        .args(["doctor", "--all", "--json"])
        .assert()
        .success();

    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    let value: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(value.as_array().unwrap().len(), 3);
    assert!(!output.contains(dir.path().to_string_lossy().as_ref()));
}

#[test]
fn base_doctor_json_does_not_print_sensitive_local_content() {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join("headsdown");
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::write(
        config_dir.join("credentials.json"),
        r#"{"api_key":"hd_secret_token_123456"}"#,
    )
    .unwrap();

    let assert = Command::cargo_bin("hd")
        .unwrap()
        .env("XDG_CONFIG_HOME", dir.path())
        .args(["--api-url", "http://127.0.0.1:9", "doctor", "--json"])
        .assert()
        .success();

    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    serde_json::from_str::<serde_json::Value>(&output).unwrap();
    assert!(!output.contains(dir.path().to_string_lossy().as_ref()));
    assert!(!output.contains("hd_secret_token"));
    assert!(!output.contains("123456"));
}

#[test]
fn doctor_claude_json_does_not_print_sensitive_local_content() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::create_dir_all(dir.path().join(".claude")).unwrap();

    let assert = Command::cargo_bin("hd")
        .unwrap()
        .env("HOME", dir.path())
        .args(["doctor", "claude", "--json"])
        .assert()
        .success();

    let output = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    assert!(!output.contains(dir.path().to_string_lossy().as_ref()));
    assert!(!output.contains("prompt"));
    assert!(!output.contains("transcript"));
    assert!(!output.contains("token"));
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
