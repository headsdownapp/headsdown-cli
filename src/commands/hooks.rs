use anyhow::{bail, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::format;

const HOOK_MARKER: &str = "# headsdown-cli managed hook";

const POST_CHECKOUT_HOOK: &str = r#"#!/bin/sh
# headsdown-cli managed hook
# Auto-sets HeadsDown mode when switching branches

# Only trigger on branch checkout (flag=1), not file checkout (flag=0)
if [ "$3" = "1" ]; then
    # Set to busy when checking out a branch (non-blocking)
    hd busy 2h 2>/dev/null &
fi
"#;

const PRE_PUSH_HOOK: &str = r#"#!/bin/sh
# headsdown-cli managed hook
# Sets HeadsDown back to online after pushing

hd online 2>/dev/null &
"#;

fn git_hooks_dir() -> Result<PathBuf> {
    // Find the git root
    let output = std::process::Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .output()?;

    if !output.status.success() {
        bail!("Not a git repository. Run this from inside a git repo");
    }

    let git_dir = String::from_utf8_lossy(&output.stdout).trim().to_string();
    Ok(PathBuf::from(git_dir).join("hooks"))
}

fn install_hook(hooks_dir: &Path, name: &str, content: &str) -> Result<bool> {
    let hook_path = hooks_dir.join(name);

    // Check if hook already exists
    if hook_path.exists() {
        let existing = fs::read_to_string(&hook_path)?;
        if existing.contains(HOOK_MARKER) {
            // Already installed by us, update it
            write_hook(&hook_path, content)?;
            return Ok(true);
        }
        // Existing hook not managed by us, don't overwrite
        println!(
            "  {} {} hook already exists (not managed by hd). Skipping",
            format::styled_yellow_bold("!"),
            name
        );
        return Ok(false);
    }

    write_hook(&hook_path, content)?;
    Ok(true)
}

fn write_hook(path: &Path, content: &str) -> Result<()> {
    fs::write(path, content)?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o755))?;
    }

    Ok(())
}

pub fn install() -> Result<()> {
    let hooks_dir = git_hooks_dir()?;
    fs::create_dir_all(&hooks_dir)?;

    println!();
    println!(
        "  {} Installing git hooks...",
        format::styled_cyan_bold("→")
    );
    println!();

    let mut installed = 0;

    if install_hook(&hooks_dir, "post-checkout", POST_CHECKOUT_HOOK)? {
        println!(
            "  {} post-checkout: auto-sets busy on branch switch",
            format::styled_green_bold("✓")
        );
        installed += 1;
    }

    if install_hook(&hooks_dir, "pre-push", PRE_PUSH_HOOK)? {
        println!(
            "  {} pre-push: auto-sets online after push",
            format::styled_green_bold("✓")
        );
        installed += 1;
    }

    println!();
    if installed > 0 {
        println!(
            "  {} {} hook(s) installed",
            format::styled_green_bold("✓"),
            installed
        );
    } else {
        println!("  No hooks were installed");
    }
    println!();

    Ok(())
}

pub fn uninstall() -> Result<()> {
    let hooks_dir = git_hooks_dir()?;

    println!();
    println!("  {} Removing git hooks...", format::styled_cyan_bold("→"));
    println!();

    let mut removed = 0;

    for name in &["post-checkout", "pre-push"] {
        let hook_path = hooks_dir.join(name);
        if hook_path.exists() {
            let content = fs::read_to_string(&hook_path)?;
            if content.contains(HOOK_MARKER) {
                fs::remove_file(&hook_path)?;
                println!("  {} Removed {}", format::styled_green_bold("✓"), name);
                removed += 1;
            }
        }
    }

    println!();
    if removed > 0 {
        println!(
            "  {} {} hook(s) removed",
            format::styled_green_bold("✓"),
            removed
        );
    } else {
        println!("  No HeadsDown hooks found to remove");
    }
    println!();

    Ok(())
}

pub fn status() -> Result<()> {
    let hooks_dir = git_hooks_dir()?;

    println!();
    println!("  {}", format::styled_bold("Git Hook Status"));
    println!();

    for name in &["post-checkout", "pre-push"] {
        let hook_path = hooks_dir.join(name);
        if hook_path.exists() {
            let content = fs::read_to_string(&hook_path)?;
            if content.contains(HOOK_MARKER) {
                println!(
                    "  {} {} (managed by hd)",
                    format::styled_green_bold("✓"),
                    name
                );
            } else {
                println!(
                    "  {} {} (exists, not managed by hd)",
                    format::styled_yellow_bold("•"),
                    name
                );
            }
        } else {
            println!("  {} {} (not installed)", format::styled_dimmed("•"), name);
        }
    }

    println!();
    Ok(())
}
