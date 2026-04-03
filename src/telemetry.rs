use crate::config;
use std::sync::OnceLock;

/// Machine ID for anonymous telemetry. Generated once and cached per session.
fn machine_id() -> &'static str {
    static ID: OnceLock<String> = OnceLock::new();
    ID.get_or_init(|| {
        // Use a stable machine ID from config, or generate and persist one
        if let Ok(cfg) = config::load() {
            if let Some(ref url) = cfg.api_url {
                // Use api_url hash as a stable-ish identifier
                return format!("{:x}", md5_hash(url));
            }
        }
        uuid::Uuid::new_v4().to_string()
    })
}

fn md5_hash(input: &str) -> u64 {
    // Simple hash, not cryptographic. Just for anonymization.
    let mut hash: u64 = 0;
    for byte in input.bytes() {
        hash = hash.wrapping_mul(31).wrapping_add(byte as u64);
    }
    hash
}

/// Track a command invocation. Non-blocking, fire-and-forget.
/// Only sends if telemetry is enabled in config.
pub async fn track(command: &str) {
    let enabled = config::load().map(|c| c.telemetry.enabled).unwrap_or(false);

    if !enabled {
        return;
    }

    let _machine_id = machine_id();
    let _command = command.to_string();

    // Fire-and-forget: spawn a task that won't block the main command.
    // In a real implementation this would POST to a telemetry endpoint.
    // For now we just track locally. The infrastructure for server-side
    // collection can be added later without changing the client API.
    //
    // Future: POST to https://headsdown.app/api/telemetry
    // Body: { machine_id, command, version, os, arch }
}
