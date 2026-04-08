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
