use crate::format;
use anyhow::Result;

pub async fn run() -> Result<()> {
    let current = env!("CARGO_PKG_VERSION");

    println!();
    println!(
        "  {} Checking for updates (current: v{})...",
        format::styled_cyan_bold("→"),
        current
    );

    // Use self_update to check GitHub Releases for the latest version
    let status = self_update::backends::github::Update::configure()
        .repo_owner("headsdownapp")
        .repo_name("headsdown-cli")
        .bin_name("hd")
        .current_version(current)
        .show_output(false)
        .show_download_progress(true)
        .no_confirm(false)
        .build()?
        .update()?;

    if status.updated() {
        println!();
        println!(
            "  {} Updated to v{}",
            format::styled_green_bold("✓"),
            status.version()
        );
    } else {
        println!(
            "  {} Already on the latest version (v{})",
            format::styled_green_bold("✓"),
            current
        );
    }
    println!();

    Ok(())
}
