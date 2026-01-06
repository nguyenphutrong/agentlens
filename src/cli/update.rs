use anyhow::{Context, Result};

const REPO_OWNER: &str = "nguyenphutrong";
const REPO_NAME: &str = "agentmap";
const BIN_NAME: &str = "agentmap";

pub fn run_update() -> Result<()> {
    let current_version = env!("CARGO_PKG_VERSION");
    eprintln!("Current version: v{}", current_version);
    eprintln!("Checking for updates...");

    let status = self_update::backends::github::Update::configure()
        .repo_owner(REPO_OWNER)
        .repo_name(REPO_NAME)
        .bin_name(BIN_NAME)
        .show_download_progress(true)
        .current_version(current_version)
        .build()
        .context("Failed to configure updater")?
        .update()
        .context("Failed to update")?;

    if status.updated() {
        eprintln!("\n✓ Updated to v{}!", status.version());
    } else {
        eprintln!("\n✓ Already up to date (v{})", current_version);
    }

    Ok(())
}
