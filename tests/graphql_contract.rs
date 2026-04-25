#[test]
fn command_queries_use_availability_root_field() {
    let status_src = include_str!("../src/commands/status.rs");
    let watch_src = include_str!("../src/commands/watch.rs");
    let availability_src = include_str!("../src/commands/availability.rs");

    assert!(
        status_src.contains("availability"),
        "status query should reference availability root field"
    );
    assert!(
        watch_src.contains("availability"),
        "watch query should reference availability root field"
    );
    assert!(
        availability_src.contains("availability"),
        "availability query should reference availability root field"
    );
}

#[test]
fn verdict_query_matches_latest_submit_proposal_shape() {
    let src = include_str!("../src/commands/verdict.rs");
    assert!(src.contains("decision"));
    assert!(src.contains("reason"));
    assert!(src.contains("proposalId"));
    assert!(src.contains("wrapUpGuidance"));
    assert!(!src.contains("policyStatus"));
}

#[test]
fn verdict_settings_query_uses_thresholds_shape() {
    let src = include_str!("../src/commands/verdict_settings.rs");
    assert!(src.contains("thresholds"));
    assert!(src.contains("defaultWrapUpMode"));
    assert!(src.contains("wrapUpThresholdMinutes"));
    assert!(!src.contains("modeThresholds"));
}

#[test]
fn windows_and_availability_handle_days_arrays() {
    let windows_src = include_str!("../src/commands/windows.rs");
    let availability_src = include_str!("../src/commands/availability.rs");
    let contract_src = include_str!("../src/contract/availability.rs");

    assert!(windows_src.contains("normalize_days_input"));
    assert!(windows_src.contains("DaysField"));
    assert!(availability_src.contains("format_days("));
    assert!(contract_src.contains("enum DaysField"));
}
