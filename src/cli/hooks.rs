use anyhow::{Context, Result};
use std::fs;
use std::path::Path;

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HookManager {
    Native,
    Husky,
    Lefthook,
    PreCommit,
}

impl std::fmt::Display for HookManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HookManager::Native => write!(f, "native git hooks"),
            HookManager::Husky => write!(f, "Husky"),
            HookManager::Lefthook => write!(f, "Lefthook"),
            HookManager::PreCommit => write!(f, "pre-commit"),
        }
    }
}

const PRE_COMMIT_HOOK: &str = r#"#!/bin/sh
# agentmap pre-commit hook

if [ -n "$AGENTMAP_SKIP" ]; then
    exit 0
fi

if command -v agentmap >/dev/null 2>&1; then
    agentmap --quiet
    git add .agentmap/ 2>/dev/null || true
fi
"#;

const POST_CHECKOUT_HOOK: &str = r#"#!/bin/sh
# agentmap post-checkout hook

if [ -n "$AGENTMAP_SKIP" ]; then
    exit 0
fi

# Only run on branch checkout (not file checkout)
if [ "$3" = "1" ]; then
    if command -v agentmap >/dev/null 2>&1; then
        agentmap --quiet &
    fi
fi
"#;

const POST_MERGE_HOOK: &str = r#"#!/bin/sh
# agentmap post-merge hook

if [ -n "$AGENTMAP_SKIP" ]; then
    exit 0
fi

if command -v agentmap >/dev/null 2>&1; then
    agentmap --quiet &
fi
"#;

pub fn detect_hook_manager(path: &Path) -> HookManager {
    if path.join(".husky").is_dir() {
        return HookManager::Husky;
    }

    if path.join("lefthook.yml").exists()
        || path.join("lefthook.yaml").exists()
        || path.join(".lefthook.yml").exists()
        || path.join(".lefthook.yaml").exists()
    {
        return HookManager::Lefthook;
    }

    if path.join(".pre-commit-config.yaml").exists() || path.join(".pre-commit-config.yml").exists()
    {
        return HookManager::PreCommit;
    }

    if let Ok(pkg_json) = fs::read_to_string(path.join("package.json")) {
        if pkg_json.contains("\"husky\"") {
            return HookManager::Husky;
        }
        if pkg_json.contains("\"lefthook\"") {
            return HookManager::Lefthook;
        }
    }

    HookManager::Native
}

pub fn install_hooks_with_manager(
    path: &Path,
    native: bool,
    husky: bool,
    lefthook: bool,
    pre_commit: bool,
) -> Result<()> {
    let manager = if native {
        HookManager::Native
    } else if husky {
        HookManager::Husky
    } else if lefthook {
        HookManager::Lefthook
    } else if pre_commit {
        HookManager::PreCommit
    } else {
        detect_hook_manager(path)
    };

    eprintln!("Detected: {} → Installing agentmap hooks", manager);

    match manager {
        HookManager::Native => install_native_hooks(path),
        HookManager::Husky => install_husky_hooks(path),
        HookManager::Lefthook => install_lefthook_hooks(path),
        HookManager::PreCommit => install_pre_commit_hooks(path),
    }
}

pub fn install_hooks(path: &Path) -> Result<()> {
    install_hooks_with_manager(path, false, false, false, false)
}

fn install_native_hooks(path: &Path) -> Result<()> {
    let git_dir = find_git_dir(path)?;
    let hooks_dir = git_dir.join("hooks");

    fs::create_dir_all(&hooks_dir).context("Failed to create hooks directory")?;

    install_native_hook(&hooks_dir, "pre-commit", PRE_COMMIT_HOOK)?;
    install_native_hook(&hooks_dir, "post-checkout", POST_CHECKOUT_HOOK)?;
    install_native_hook(&hooks_dir, "post-merge", POST_MERGE_HOOK)?;

    eprintln!("Installed agentmap git hooks:");
    eprintln!("  - pre-commit: regenerate docs and stage .agentmap/");
    eprintln!("  - post-checkout: regenerate docs after branch switch");
    eprintln!("  - post-merge: regenerate docs after pull/merge");
    eprintln!("\nTo skip hooks, set AGENTMAP_SKIP=1");

    Ok(())
}

fn install_husky_hooks(path: &Path) -> Result<()> {
    let husky_dir = path.join(".husky");

    if !husky_dir.exists() {
        fs::create_dir_all(&husky_dir).context("Failed to create .husky directory")?;
    }

    let pre_commit_content = r#"#!/bin/sh

if [ -n "$AGENTMAP_SKIP" ]; then
    exit 0
fi

if command -v agentmap >/dev/null 2>&1; then
    agentmap --quiet
    git add .agentmap/ 2>/dev/null || true
elif command -v npx >/dev/null 2>&1; then
    npx agentmap-cli --quiet
    git add .agentmap/ 2>/dev/null || true
fi
"#;

    let post_checkout_content = r#"#!/bin/sh

if [ -n "$AGENTMAP_SKIP" ]; then
    exit 0
fi

# Only run on branch checkout (not file checkout)
if [ "$3" = "1" ]; then
    if command -v agentmap >/dev/null 2>&1; then
        agentmap --quiet &
    elif command -v npx >/dev/null 2>&1; then
        npx agentmap-cli --quiet &
    fi
fi
"#;

    let post_merge_content = r#"#!/bin/sh

if [ -n "$AGENTMAP_SKIP" ]; then
    exit 0
fi

if command -v agentmap >/dev/null 2>&1; then
    agentmap --quiet &
elif command -v npx >/dev/null 2>&1; then
    npx agentmap-cli --quiet &
fi
"#;

    install_husky_hook(&husky_dir, "pre-commit", pre_commit_content)?;
    install_husky_hook(&husky_dir, "post-checkout", post_checkout_content)?;
    install_husky_hook(&husky_dir, "post-merge", post_merge_content)?;

    eprintln!("Installed agentmap Husky hooks:");
    eprintln!("  - .husky/pre-commit");
    eprintln!("  - .husky/post-checkout");
    eprintln!("  - .husky/post-merge");
    eprintln!("\nTo skip hooks, set AGENTMAP_SKIP=1");

    Ok(())
}

fn install_husky_hook(husky_dir: &Path, name: &str, content: &str) -> Result<()> {
    let hook_path = husky_dir.join(name);

    if hook_path.exists() {
        let existing = fs::read_to_string(&hook_path).unwrap_or_default();
        if existing.contains("agentmap") {
            eprintln!("  .husky/{} already contains agentmap, skipping", name);
            return Ok(());
        }

        let combined = format!(
            "{}\n\n# --- agentmap ---\n{}",
            existing.trim(),
            content.lines().skip(1).collect::<Vec<_>>().join("\n")
        );
        fs::write(&hook_path, combined).context(format!("Failed to update .husky/{}", name))?;
        eprintln!("  .husky/{} updated (appended)", name);
    } else {
        fs::write(&hook_path, content).context(format!("Failed to create .husky/{}", name))?;
        eprintln!("  .husky/{} created", name);
    }

    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms)?;
    }

    Ok(())
}

fn install_lefthook_hooks(path: &Path) -> Result<()> {
    let config_files = [
        "lefthook.yml",
        "lefthook.yaml",
        ".lefthook.yml",
        ".lefthook.yaml",
    ];

    let config_path = config_files
        .iter()
        .map(|f| path.join(f))
        .find(|p| p.exists())
        .unwrap_or_else(|| path.join("lefthook.yml"));

    let agentmap_config = r#"
# --- agentmap hooks ---
pre-commit:
  commands:
    agentmap:
      run: |
        if [ -z "$AGENTMAP_SKIP" ]; then
          if command -v agentmap >/dev/null 2>&1; then
            agentmap --quiet && git add .agentmap/ 2>/dev/null || true
          elif command -v npx >/dev/null 2>&1; then
            npx agentmap-cli --quiet && git add .agentmap/ 2>/dev/null || true
          fi
        fi
      stage_fixed: true

post-checkout:
  commands:
    agentmap:
      run: |
        if [ -z "$AGENTMAP_SKIP" ] && [ "$LEFTHOOK_GIT_CHECKOUT_TYPE" = "branch" ]; then
          if command -v agentmap >/dev/null 2>&1; then
            agentmap --quiet &
          elif command -v npx >/dev/null 2>&1; then
            npx agentmap-cli --quiet &
          fi
        fi

post-merge:
  commands:
    agentmap:
      run: |
        if [ -z "$AGENTMAP_SKIP" ]; then
          if command -v agentmap >/dev/null 2>&1; then
            agentmap --quiet &
          elif command -v npx >/dev/null 2>&1; then
            npx agentmap-cli --quiet &
          fi
        fi
"#;

    if config_path.exists() {
        let existing = fs::read_to_string(&config_path).unwrap_or_default();
        if existing.contains("agentmap") {
            eprintln!(
                "  {} already contains agentmap, skipping",
                config_path.display()
            );
            return Ok(());
        }

        let combined = format!("{}\n{}", existing.trim(), agentmap_config);
        fs::write(&config_path, combined)
            .context(format!("Failed to update {}", config_path.display()))?;
        eprintln!("  {} updated", config_path.display());
    } else {
        let content = format!(
            "# Lefthook configuration\n# https://github.com/evilmartians/lefthook\n{}",
            agentmap_config
        );
        fs::write(&config_path, content)
            .context(format!("Failed to create {}", config_path.display()))?;
        eprintln!("  {} created", config_path.display());
    }

    eprintln!("\nInstalled agentmap Lefthook hooks.");
    eprintln!("Run 'lefthook install' to activate hooks.");
    eprintln!("\nTo skip hooks, set AGENTMAP_SKIP=1");

    Ok(())
}

fn install_pre_commit_hooks(path: &Path) -> Result<()> {
    let config_files = [".pre-commit-config.yaml", ".pre-commit-config.yml"];

    let config_path = config_files
        .iter()
        .map(|f| path.join(f))
        .find(|p| p.exists())
        .unwrap_or_else(|| path.join(".pre-commit-config.yaml"));

    let agentmap_repo = r#"
  # --- agentmap ---
  - repo: local
    hooks:
      - id: agentmap
        name: agentmap
        entry: sh -c 'if [ -z "$AGENTMAP_SKIP" ]; then if command -v agentmap >/dev/null 2>&1; then agentmap --quiet && git add .agentmap/; elif command -v npx >/dev/null 2>&1; then npx agentmap-cli --quiet && git add .agentmap/; fi; fi'
        language: system
        always_run: true
        pass_filenames: false
        stages: [pre-commit]
"#;

    if config_path.exists() {
        let existing = fs::read_to_string(&config_path).unwrap_or_default();
        if existing.contains("agentmap") {
            eprintln!(
                "  {} already contains agentmap, skipping",
                config_path.display()
            );
            return Ok(());
        }

        let combined = format!("{}\n{}", existing.trim(), agentmap_repo);
        fs::write(&config_path, combined)
            .context(format!("Failed to update {}", config_path.display()))?;
        eprintln!("  {} updated", config_path.display());
    } else {
        let content = format!(
            "# Pre-commit configuration\n# https://pre-commit.com\nrepos:{}\n",
            agentmap_repo
        );
        fs::write(&config_path, content)
            .context(format!("Failed to create {}", config_path.display()))?;
        eprintln!("  {} created", config_path.display());
    }

    eprintln!("\nInstalled agentmap pre-commit hook.");
    eprintln!("Run 'pre-commit install' to activate hooks.");
    eprintln!("\nNote: post-checkout and post-merge hooks are not supported by pre-commit.");
    eprintln!("Consider using 'agentmap hooks install --native' for full hook support.");
    eprintln!("\nTo skip hooks, set AGENTMAP_SKIP=1");

    Ok(())
}

pub fn remove_hooks(path: &Path) -> Result<()> {
    let manager = detect_hook_manager(path);

    eprintln!("Detected: {} → Removing agentmap hooks", manager);

    match manager {
        HookManager::Native => remove_native_hooks(path),
        HookManager::Husky => remove_husky_hooks(path),
        HookManager::Lefthook => remove_lefthook_hooks(path),
        HookManager::PreCommit => remove_pre_commit_hooks(path),
    }
}

fn remove_native_hooks(path: &Path) -> Result<()> {
    let git_dir = find_git_dir(path)?;
    let hooks_dir = git_dir.join("hooks");

    remove_native_hook(&hooks_dir, "pre-commit")?;
    remove_native_hook(&hooks_dir, "post-checkout")?;
    remove_native_hook(&hooks_dir, "post-merge")?;

    eprintln!("Removed agentmap git hooks");

    Ok(())
}

fn remove_husky_hooks(path: &Path) -> Result<()> {
    let husky_dir = path.join(".husky");

    for name in ["pre-commit", "post-checkout", "post-merge"] {
        let hook_path = husky_dir.join(name);
        if hook_path.exists() {
            let content = fs::read_to_string(&hook_path).unwrap_or_default();
            if content.contains("agentmap") {
                if content.contains("# --- agentmap ---") {
                    let cleaned = content
                        .split("# --- agentmap ---")
                        .next()
                        .unwrap_or("")
                        .trim();
                    if cleaned.is_empty() || cleaned == "#!/bin/sh" {
                        fs::remove_file(&hook_path)?;
                        eprintln!("  .husky/{} removed", name);
                    } else {
                        fs::write(&hook_path, cleaned)?;
                        eprintln!("  .husky/{} updated (agentmap section removed)", name);
                    }
                } else {
                    fs::remove_file(&hook_path)?;
                    eprintln!("  .husky/{} removed", name);
                }
            }
        }
    }

    eprintln!("Removed agentmap Husky hooks");

    Ok(())
}

fn remove_lefthook_hooks(path: &Path) -> Result<()> {
    let config_files = [
        "lefthook.yml",
        "lefthook.yaml",
        ".lefthook.yml",
        ".lefthook.yaml",
    ];

    for config_file in config_files {
        let config_path = path.join(config_file);
        if config_path.exists() {
            let content = fs::read_to_string(&config_path).unwrap_or_default();
            if content.contains("# --- agentmap hooks ---") {
                let cleaned = content
                    .split("# --- agentmap hooks ---")
                    .next()
                    .unwrap_or("")
                    .trim();
                fs::write(&config_path, format!("{}\n", cleaned))?;
                eprintln!("  {} updated (agentmap section removed)", config_file);
            }
        }
    }

    eprintln!("Removed agentmap Lefthook hooks");

    Ok(())
}

fn remove_pre_commit_hooks(path: &Path) -> Result<()> {
    let config_files = [".pre-commit-config.yaml", ".pre-commit-config.yml"];

    for config_file in config_files {
        let config_path = path.join(config_file);
        if config_path.exists() {
            let content = fs::read_to_string(&config_path).unwrap_or_default();
            if content.contains("# --- agentmap ---") {
                let cleaned = content
                    .split("# --- agentmap ---")
                    .next()
                    .unwrap_or("")
                    .trim();
                fs::write(&config_path, format!("{}\n", cleaned))?;
                eprintln!("  {} updated (agentmap section removed)", config_file);
            }
        }
    }

    eprintln!("Removed agentmap pre-commit hooks");

    Ok(())
}

fn find_git_dir(path: &Path) -> Result<std::path::PathBuf> {
    let mut current = path.to_path_buf();
    loop {
        let git_dir = current.join(".git");
        if git_dir.is_dir() {
            return Ok(git_dir);
        }
        if !current.pop() {
            break;
        }
    }
    anyhow::bail!("Not a git repository (or any parent)")
}

fn install_native_hook(hooks_dir: &Path, name: &str, content: &str) -> Result<()> {
    let hook_path = hooks_dir.join(name);

    if hook_path.exists() {
        let existing = fs::read_to_string(&hook_path).unwrap_or_default();
        if existing.contains("agentmap") {
            eprintln!("  {} hook already contains agentmap, skipping", name);
            return Ok(());
        }

        let combined = format!(
            "{}\n\n# --- agentmap hook ---\n{}",
            existing.trim(),
            content.trim()
        );
        fs::write(&hook_path, combined).context(format!("Failed to update {} hook", name))?;
        eprintln!("  {} hook updated (appended)", name);
    } else {
        fs::write(&hook_path, content).context(format!("Failed to create {} hook", name))?;
        eprintln!("  {} hook created", name);
    }

    #[cfg(unix)]
    {
        let mut perms = fs::metadata(&hook_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&hook_path, perms)?;
    }

    Ok(())
}

fn remove_native_hook(hooks_dir: &Path, name: &str) -> Result<()> {
    let hook_path = hooks_dir.join(name);

    if !hook_path.exists() {
        return Ok(());
    }

    let content = fs::read_to_string(&hook_path).unwrap_or_default();

    if !content.contains("agentmap") {
        return Ok(());
    }

    if content.contains("# --- agentmap hook ---") {
        let cleaned: Vec<&str> = content
            .split("# --- agentmap hook ---")
            .next()
            .unwrap_or("")
            .trim()
            .lines()
            .collect();

        if cleaned.is_empty() || (cleaned.len() == 1 && cleaned[0] == "#!/bin/sh") {
            fs::remove_file(&hook_path).context(format!("Failed to remove {} hook", name))?;
            eprintln!("  {} hook removed", name);
        } else {
            fs::write(&hook_path, cleaned.join("\n"))
                .context(format!("Failed to update {} hook", name))?;
            eprintln!("  {} hook updated (agentmap section removed)", name);
        }
    } else {
        fs::remove_file(&hook_path).context(format!("Failed to remove {} hook", name))?;
        eprintln!("  {} hook removed", name);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_find_git_dir() {
        let temp = TempDir::new().unwrap();
        let git_dir = temp.path().join(".git");
        fs::create_dir(&git_dir).unwrap();

        let result = find_git_dir(temp.path());
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), git_dir);
    }

    #[test]
    fn test_find_git_dir_not_found() {
        let temp = TempDir::new().unwrap();
        let result = find_git_dir(temp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_detect_native() {
        let temp = TempDir::new().unwrap();
        assert_eq!(detect_hook_manager(temp.path()), HookManager::Native);
    }

    #[test]
    fn test_detect_husky() {
        let temp = TempDir::new().unwrap();
        fs::create_dir(temp.path().join(".husky")).unwrap();
        assert_eq!(detect_hook_manager(temp.path()), HookManager::Husky);
    }

    #[test]
    fn test_detect_lefthook() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join("lefthook.yml"), "").unwrap();
        assert_eq!(detect_hook_manager(temp.path()), HookManager::Lefthook);
    }

    #[test]
    fn test_detect_pre_commit() {
        let temp = TempDir::new().unwrap();
        fs::write(temp.path().join(".pre-commit-config.yaml"), "").unwrap();
        assert_eq!(detect_hook_manager(temp.path()), HookManager::PreCommit);
    }
}
