use serde::Serialize;
use std::collections::HashMap;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize)]
pub struct DiffStat {
    pub path: String,
    pub status: DiffStatus,
    pub additions: usize,
    pub deletions: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum DiffStatus {
    Added,
    Modified,
    Deleted,
    Renamed,
}

impl DiffStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            DiffStatus::Added => "new",
            DiffStatus::Modified => "modified",
            DiffStatus::Deleted => "deleted",
            DiffStatus::Renamed => "renamed",
        }
    }
}

pub fn is_git_repo(path: &Path) -> bool {
    Command::new("git")
        .args(["rev-parse", "--git-dir"])
        .current_dir(path)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn get_git_head(path: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short=7", "HEAD"])
        .current_dir(path)
        .output()
        .ok()?;

    if output.status.success() {
        Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        None
    }
}

pub fn get_default_branch(path: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["symbolic-ref", "refs/remotes/origin/HEAD", "--short"])
        .current_dir(path)
        .output()
        .ok()?;

    if output.status.success() {
        let branch = String::from_utf8_lossy(&output.stdout);
        return Some(branch.trim().replace("origin/", ""));
    }

    for branch in ["main", "master"] {
        let check = Command::new("git")
            .args(["rev-parse", "--verify", branch])
            .current_dir(path)
            .output()
            .ok()?;
        if check.status.success() {
            return Some(branch.to_string());
        }
    }

    None
}

pub fn get_diff_files(path: &Path, base_ref: &str) -> Option<Vec<DiffStat>> {
    let ref_to_use = resolve_ref(path, base_ref);

    let numstat = Command::new("git")
        .args(["diff", "--numstat", &format!("{}...HEAD", ref_to_use)])
        .current_dir(path)
        .output()
        .ok()?;

    let name_status = Command::new("git")
        .args(["diff", "--name-status", &format!("{}...HEAD", ref_to_use)])
        .current_dir(path)
        .output()
        .ok()?;

    if !numstat.status.success() || !name_status.status.success() {
        return None;
    }

    let mut stats: HashMap<String, (usize, usize)> = HashMap::new();
    for line in String::from_utf8_lossy(&numstat.stdout).lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 3 {
            let adds = parts[0].parse().unwrap_or(0);
            let dels = parts[1].parse().unwrap_or(0);
            let file = parts[2].to_string();
            stats.insert(file, (adds, dels));
        }
    }

    let mut results = Vec::new();
    for line in String::from_utf8_lossy(&name_status.stdout).lines() {
        let parts: Vec<&str> = line.split('\t').collect();
        if parts.len() >= 2 {
            let status = match parts[0].chars().next() {
                Some('A') => DiffStatus::Added,
                Some('M') => DiffStatus::Modified,
                Some('D') => DiffStatus::Deleted,
                Some('R') => DiffStatus::Renamed,
                _ => DiffStatus::Modified,
            };
            let file = if status == DiffStatus::Renamed && parts.len() >= 3 {
                parts[2].to_string()
            } else {
                parts[1].to_string()
            };
            let (additions, deletions) = stats.get(&file).copied().unwrap_or((0, 0));
            results.push(DiffStat {
                path: file,
                status,
                additions,
                deletions,
            });
        }
    }

    Some(results)
}

fn resolve_ref(path: &Path, base_ref: &str) -> String {
    if base_ref.starts_with("origin/") {
        return base_ref.to_string();
    }

    let origin_ref = format!("origin/{}", base_ref);
    let check = Command::new("git")
        .args(["rev-parse", "--verify", &origin_ref])
        .current_dir(path)
        .output();

    if check.map(|o| o.status.success()).unwrap_or(false) {
        return origin_ref;
    }

    base_ref.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_status_as_str() {
        assert_eq!(DiffStatus::Added.as_str(), "new");
        assert_eq!(DiffStatus::Modified.as_str(), "modified");
        assert_eq!(DiffStatus::Deleted.as_str(), "deleted");
    }
}
