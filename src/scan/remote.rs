use anyhow::{Context, Result};
use std::path::PathBuf;
use std::process::Command;

pub fn is_remote_url(path: &str) -> bool {
    path.starts_with("https://github.com")
        || path.starts_with("https://gitlab.com")
        || path.starts_with("github.com")
        || path.starts_with("gitlab.com")
        || path.starts_with("git@github.com")
        || path.starts_with("git@gitlab.com")
}

pub fn normalize_git_url(path: &str) -> String {
    let url = if path.starts_with("github.com") || path.starts_with("gitlab.com") {
        format!("https://{}", path)
    } else {
        path.to_string()
    };

    if url.ends_with(".git") {
        url
    } else {
        format!("{}.git", url)
    }
}

pub fn clone_to_temp(url: &str) -> Result<PathBuf> {
    let temp_dir = std::env::temp_dir().join(format!("agentmap-{}", std::process::id()));
    std::fs::create_dir_all(&temp_dir).context("Failed to create temp directory")?;

    let git_url = normalize_git_url(url);

    let output = Command::new("git")
        .args([
            "clone",
            "--depth",
            "1",
            "--single-branch",
            &git_url,
            temp_dir.to_str().unwrap(),
        ])
        .output()
        .context("Failed to run git clone")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("Git clone failed: {}", stderr));
    }

    Ok(temp_dir)
}

pub fn cleanup_temp(path: &PathBuf) {
    let _ = std::fs::remove_dir_all(path);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_remote_url() {
        assert!(is_remote_url("https://github.com/user/repo"));
        assert!(is_remote_url("github.com/user/repo"));
        assert!(is_remote_url("https://gitlab.com/user/repo"));
        assert!(!is_remote_url("."));
        assert!(!is_remote_url("/path/to/local"));
    }

    #[test]
    fn test_normalize_git_url() {
        assert_eq!(
            normalize_git_url("github.com/user/repo"),
            "https://github.com/user/repo.git"
        );
        assert_eq!(
            normalize_git_url("https://github.com/user/repo"),
            "https://github.com/user/repo.git"
        );
        assert_eq!(
            normalize_git_url("https://github.com/user/repo.git"),
            "https://github.com/user/repo.git"
        );
    }
}
