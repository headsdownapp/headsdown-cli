use assert_cmd::Command;
use serde_json::Value;
use tempfile::TempDir;
use wiremock::matchers::{body_string_contains, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn prepare_auth_dir() -> TempDir {
    let dir = tempfile::tempdir().unwrap();
    let config_dir = dir.path().join("headsdown");
    std::fs::create_dir_all(&config_dir).unwrap();
    std::fs::write(
        config_dir.join("credentials.json"),
        r#"{"apiKey":"hd_test_token","createdAt":"2026-04-21T00:00:00Z"}"#,
    )
    .unwrap();
    dir
}

fn run_json(args: &[&str], auth_dir: &TempDir) -> Value {
    let assert = Command::cargo_bin("hd")
        .unwrap()
        .args(args)
        .env("XDG_CONFIG_HOME", auth_dir.path())
        .assert()
        .success();

    let stdout = String::from_utf8(assert.get_output().stdout.clone()).unwrap();
    serde_json::from_str(&stdout).unwrap()
}

#[tokio::test]
async fn verdict_json_matches_latest_submit_proposal_shape() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .and(body_string_contains("mutation SubmitProposal"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "submitProposal": {
                    "decision": "APPROVED",
                    "reason": "Looks good",
                    "proposalId": "prop_123",
                    "evaluatedAt": "2026-04-21T16:00:00Z",
                    "wrapUpGuidance": {
                        "active": false,
                        "deadlineAt": null,
                        "remainingMinutes": null,
                        "profile": "NORMAL",
                        "source": "INACTIVE",
                        "reason": "Outside threshold",
                        "hints": [],
                        "thresholdMinutes": 30,
                        "selectedMode": "AUTO"
                    }
                }
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let auth_dir = prepare_auth_dir();
    let json = run_json(
        &[
            "--api-url",
            &server.uri(),
            "verdict",
            "refactor auth module",
            "--files",
            "5",
            "--minutes",
            "30",
            "--json",
        ],
        &auth_dir,
    );

    assert_eq!(json["decision"], "APPROVED");
    assert_eq!(json["proposalId"], "prop_123");
    assert_eq!(json["wrapUpGuidance"]["selectedMode"], "AUTO");
}

#[tokio::test]
async fn verdict_settings_get_json_matches_thresholds_shape() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .and(body_string_contains("query"))
        .and(body_string_contains("verdictSettings"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "verdictSettings": {
                    "id": "vs_1",
                    "thresholds": {
                        "online": {"maxFiles": 10, "maxEstimatedMinutes": 120},
                        "busy": {"maxFiles": 3, "maxEstimatedMinutes": 45},
                        "limited": {"maxFiles": 2, "maxEstimatedMinutes": 30},
                        "offline": {"maxFiles": 0, "maxEstimatedMinutes": 0}
                    },
                    "defaultWrapUpMode": "AUTO",
                    "wrapUpThresholdMinutes": 30,
                    "updatedAt": "2026-04-21T16:00:00Z"
                }
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let auth_dir = prepare_auth_dir();
    let json = run_json(
        &[
            "--api-url",
            &server.uri(),
            "verdict-settings",
            "get",
            "--json",
        ],
        &auth_dir,
    );

    assert_eq!(json["id"], "vs_1");
    assert_eq!(json["thresholds"]["busy"]["maxFiles"], 3);
    assert_eq!(json["defaultWrapUpMode"], "AUTO");
}

#[tokio::test]
async fn verdict_settings_set_sends_new_shape_and_returns_payload() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .and(body_string_contains("mutation UpdateVerdictSettings"))
        .and(body_string_contains("\"thresholds\""))
        .and(body_string_contains("\"defaultWrapUpMode\":\"WRAP_UP\""))
        .and(body_string_contains("\"wrapUpThresholdMinutes\":25"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "updateVerdictSettings": {
                    "id": "vs_2",
                    "thresholds": {
                        "online": {"maxFiles": 8, "maxEstimatedMinutes": 90},
                        "busy": {"maxFiles": 3, "maxEstimatedMinutes": 30},
                        "limited": {"maxFiles": 2, "maxEstimatedMinutes": 20},
                        "offline": {"maxFiles": 0, "maxEstimatedMinutes": 0}
                    },
                    "defaultWrapUpMode": "WRAP_UP",
                    "wrapUpThresholdMinutes": 25,
                    "updatedAt": "2026-04-21T16:05:00Z"
                }
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let auth_dir = prepare_auth_dir();
    let thresholds = r#"{"online":{"maxFiles":8,"maxEstimatedMinutes":90}}"#;
    let json = run_json(
        &[
            "--api-url",
            &server.uri(),
            "verdict-settings",
            "set",
            "--thresholds",
            thresholds,
            "--default-wrap-up-mode",
            "wrap_up",
            "--wrap-up-threshold-minutes",
            "25",
            "--json",
        ],
        &auth_dir,
    );

    assert_eq!(json["id"], "vs_2");
    assert_eq!(json["defaultWrapUpMode"], "WRAP_UP");
    assert_eq!(json["wrapUpThresholdMinutes"], 25);
}

#[tokio::test]
async fn grants_list_active_json_works() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .and(body_string_contains("query ActiveDelegationGrants"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "activeDelegationGrants": [
                    {
                        "id": "grant_1",
                        "scope": "WORKSPACE",
                        "sessionId": null,
                        "workspaceRef": "/repo",
                        "agentId": "pi-agent",
                        "permissions": ["PRESET_APPLY"],
                        "source": "pi",
                        "expiresAt": "2026-04-21T20:00:00Z",
                        "revokedAt": null,
                        "expiredAt": null,
                        "insertedAt": "2026-04-21T15:00:00Z"
                    }
                ]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let auth_dir = prepare_auth_dir();
    let json = run_json(
        &[
            "--api-url",
            &server.uri(),
            "grants",
            "list-active",
            "--json",
        ],
        &auth_dir,
    );

    assert_eq!(json.as_array().unwrap().len(), 1);
    assert_eq!(json[0]["id"], "grant_1");
}

#[tokio::test]
async fn grants_create_and_revoke_many_mutations_work() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .and(body_string_contains("mutation CreateDelegationGrant"))
        .and(body_string_contains("\"scope\":\"WORKSPACE\""))
        .and(body_string_contains(
            "\"permissions\":[\"PRESET_APPLY\",\"AVAILABILITY_OVERRIDE_CREATE\"]",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "createDelegationGrant": {
                    "id": "grant_2",
                    "scope": "WORKSPACE",
                    "sessionId": null,
                    "workspaceRef": "/repo",
                    "agentId": null,
                    "permissions": ["PRESET_APPLY", "AVAILABILITY_OVERRIDE_CREATE"],
                    "source": "hd",
                    "expiresAt": "2026-04-21T20:00:00Z",
                    "revokedAt": null,
                    "expiredAt": null,
                    "insertedAt": "2026-04-21T16:10:00Z"
                }
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .and(body_string_contains("mutation RevokeDelegationGrants"))
        .and(body_string_contains("\"scope\":\"WORKSPACE\""))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "revokeDelegationGrants": {
                    "revokedCount": 2
                }
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let auth_dir = prepare_auth_dir();
    let created = run_json(
        &[
            "--api-url",
            &server.uri(),
            "grants",
            "create",
            "--scope",
            "workspace",
            "--workspace-ref",
            "/repo",
            "--permissions",
            "preset_apply,availability_override_create",
            "--json",
        ],
        &auth_dir,
    );
    assert_eq!(created["id"], "grant_2");

    let revoked = run_json(
        &[
            "--api-url",
            &server.uri(),
            "grants",
            "revoke-many",
            "--scope",
            "workspace",
            "--json",
        ],
        &auth_dir,
    );
    assert_eq!(revoked["revokedCount"], 2);
}

#[tokio::test]
async fn override_get_set_clear_json_work() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .and(body_string_contains("query ActiveAvailabilityOverride"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "activeAvailabilityOverride": {
                    "id": "ovr_1",
                    "mode": "BUSY",
                    "reason": "focus",
                    "source": "pi",
                    "expiresAt": "2026-04-21T18:00:00Z",
                    "cancelledAt": null,
                    "expiredAt": null,
                    "insertedAt": "2026-04-21T16:00:00Z",
                    "updatedAt": "2026-04-21T16:00:00Z"
                }
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .and(body_string_contains("mutation CreateAvailabilityOverride"))
        .and(body_string_contains("\"mode\":\"BUSY\""))
        .and(body_string_contains("\"durationMinutes\":30"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "createAvailabilityOverride": {
                    "id": "ovr_2",
                    "mode": "BUSY",
                    "reason": "focus",
                    "source": "hd",
                    "expiresAt": "2026-04-21T17:00:00Z",
                    "cancelledAt": null,
                    "expiredAt": null,
                    "insertedAt": "2026-04-21T16:20:00Z",
                    "updatedAt": "2026-04-21T16:20:00Z"
                }
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/graphql"))
        .and(body_string_contains("mutation CancelAvailabilityOverride"))
        .and(body_string_contains("\"id\":\"ovr_2\""))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "cancelAvailabilityOverride": {
                    "id": "ovr_2",
                    "mode": "BUSY",
                    "reason": "done",
                    "source": "hd",
                    "expiresAt": "2026-04-21T17:00:00Z",
                    "cancelledAt": "2026-04-21T16:30:00Z",
                    "expiredAt": null,
                    "insertedAt": "2026-04-21T16:20:00Z",
                    "updatedAt": "2026-04-21T16:30:00Z"
                }
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let auth_dir = prepare_auth_dir();

    let current = run_json(
        &["--api-url", &server.uri(), "override", "get", "--json"],
        &auth_dir,
    );
    assert_eq!(current["id"], "ovr_1");

    let created = run_json(
        &[
            "--api-url",
            &server.uri(),
            "override",
            "set",
            "--mode",
            "busy",
            "--duration-minutes",
            "30",
            "--reason",
            "focus",
            "--json",
        ],
        &auth_dir,
    );
    assert_eq!(created["id"], "ovr_2");

    let cleared = run_json(
        &[
            "--api-url",
            &server.uri(),
            "override",
            "clear",
            "--id",
            "ovr_2",
            "--reason",
            "done",
            "--json",
        ],
        &auth_dir,
    );
    assert_eq!(cleared["id"], "ovr_2");
    assert_eq!(cleared["cancelledAt"], "2026-04-21T16:30:00Z");
}

#[tokio::test]
async fn migrated_commands_fail_fast_on_shape_mismatch() {
    struct Case {
        operation_hint: &'static str,
        args: Vec<&'static str>,
        response_data: Value,
    }

    let cases = vec![
        Case {
            operation_hint: "activeContract",
            args: vec!["status", "--json"],
            response_data: serde_json::json!({
                "activeContract": {"mode": 123, "statusText": null, "statusEmoji": null, "expiresAt": null, "lock": false},
                "availability": null,
                "profile": null
            }),
        },
        Case {
            operation_hint: "availability",
            args: vec!["availability", "--json"],
            response_data: serde_json::json!({
                "availability": {"inReachableHours": "yes", "nextTransitionAt": null, "activeWindow": null, "nextWindow": null}
            }),
        },
        Case {
            operation_hint: "reachabilityWindows",
            args: vec!["windows", "list", "--json"],
            response_data: serde_json::json!({
                "reachabilityWindows": [{"id":"w1","label":"Focus","mode":7,"days":["MONDAY"],"startTime":"09:00:00","endTime":"17:00:00","alertsPolicy":"OFF","autoActivate":true,"priority":1,"status":false,"statusEmoji":null,"statusText":null,"snooze":false}]
            }),
        },
        Case {
            operation_hint: "presets",
            args: vec!["presets", "list", "--json"],
            response_data: serde_json::json!({
                "presets": [{"id":"p1","name":99,"statusEmoji":null,"statusText":"Deep work","duration":30}]
            }),
        },
        Case {
            operation_hint: "activeDelegationGrants",
            args: vec!["grants", "list-active", "--json"],
            response_data: serde_json::json!({
                "activeDelegationGrants": [{"id":"g1","scope":42,"expiresAt":null,"permissions":[]}]
            }),
        },
        Case {
            operation_hint: "activeAvailabilityOverride",
            args: vec!["override", "get", "--json"],
            response_data: serde_json::json!({
                "activeAvailabilityOverride": {"id":"ovr_1","mode":1,"expiresAt":null,"cancelledAt":null}
            }),
        },
        Case {
            operation_hint: "submitProposal",
            args: vec![
                "verdict",
                "refactor auth",
                "--files",
                "3",
                "--minutes",
                "20",
                "--json",
            ],
            response_data: serde_json::json!({
                "submitProposal": {"decision":1,"reason":"ok","proposalId":"prop_1","wrapUpGuidance":null}
            }),
        },
        Case {
            operation_hint: "verdictSettings",
            args: vec!["verdict-settings", "get", "--json"],
            response_data: serde_json::json!({
                "verdictSettings": {"id":"vs_1","thresholds":{},"defaultWrapUpMode":1,"wrapUpThresholdMinutes":30,"updatedAt":"2026-04-21T16:00:00Z"}
            }),
        },
    ];

    for case in cases {
        let server = MockServer::start().await;

        Mock::given(method("POST"))
            .and(path("/graphql"))
            .and(body_string_contains(case.operation_hint))
            .respond_with(
                ResponseTemplate::new(200)
                    .set_body_json(serde_json::json!({"data": case.response_data})),
            )
            .expect(1)
            .mount(&server)
            .await;

        let auth_dir = prepare_auth_dir();
        let mut args: Vec<String> = vec!["--api-url".to_string(), server.uri()];
        args.extend(case.args.iter().map(|value| value.to_string()));

        let assert = Command::cargo_bin("hd")
            .unwrap()
            .args(&args)
            .env("XDG_CONFIG_HOME", auth_dir.path())
            .assert()
            .failure();
        let stderr = String::from_utf8(assert.get_output().stderr.clone()).unwrap();
        assert!(
            stderr.contains("Failed to decode API response shape"),
            "expected decode-shape failure for operation hint {}, got: {}",
            case.operation_hint,
            stderr
        );
    }
}
