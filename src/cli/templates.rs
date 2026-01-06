use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::generate::{generate_template, parse_template_types, TemplateConfig, TemplateType};

const AGENTMAP_MARKER: &str = "# Agentmap Integration";
const AGENTMAP_MARKER_ALT: &str = "## Agentmap Documentation";

pub fn run_templates(path: &Path, templates_arg: Option<String>, agentmap_dir: &str) -> Result<()> {
    let template_types = match templates_arg {
        Some(ref s) if !s.is_empty() => parse_template_types(s)
            .ok_or_else(|| anyhow::anyhow!("Invalid template type(s): {}", s))?,
        _ => TemplateType::all(),
    };

    let config = TemplateConfig {
        project_name: infer_project_name(path),
        agentmap_dir,
    };

    let mut created_count = 0;
    let mut appended_count = 0;
    let mut skipped_count = 0;

    for template_type in &template_types {
        let filename = template_type.filename();
        let file_path = path.join(filename);
        let content = generate_template(*template_type, &config);

        match write_template_file(&file_path, &content) {
            TemplateWriteResult::Created => {
                eprintln!("Created: {}", file_path.display());
                created_count += 1;
            }
            TemplateWriteResult::Appended => {
                eprintln!("Appended to: {}", file_path.display());
                appended_count += 1;
            }
            TemplateWriteResult::Skipped(reason) => {
                eprintln!("Skipped {}: {}", filename, reason);
                skipped_count += 1;
            }
            TemplateWriteResult::Error(e) => {
                eprintln!("Error writing {}: {}", filename, e);
            }
        }
    }

    if created_count + appended_count > 0 {
        eprintln!(
            "\nTemplates: {} created, {} appended, {} skipped",
            created_count, appended_count, skipped_count
        );
    }

    Ok(())
}

enum TemplateWriteResult {
    Created,
    Appended,
    Skipped(String),
    Error(String),
}

fn write_template_file(path: &Path, content: &str) -> TemplateWriteResult {
    if path.exists() {
        let existing = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => return TemplateWriteResult::Error(e.to_string()),
        };

        if existing.contains(AGENTMAP_MARKER) || existing.contains(AGENTMAP_MARKER_ALT) {
            return TemplateWriteResult::Skipped("agentmap section already exists".to_string());
        }

        let new_content = format!("{}\n\n{}", existing.trim_end(), content);
        match fs::write(path, new_content) {
            Ok(()) => TemplateWriteResult::Appended,
            Err(e) => TemplateWriteResult::Error(e.to_string()),
        }
    } else {
        match fs::write(path, content) {
            Ok(()) => TemplateWriteResult::Created,
            Err(e) => TemplateWriteResult::Error(e.to_string()),
        }
    }
}

fn infer_project_name(path: &Path) -> Option<&'static str> {
    let _ = path;
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_write_template_creates_new_file() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join(".cursorrules");

        let result = write_template_file(&file_path, "# Test content");

        assert!(matches!(result, TemplateWriteResult::Created));
        assert!(file_path.exists());
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("# Test content"));
    }

    #[test]
    fn test_write_template_appends_to_existing() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join(".cursorrules");

        fs::write(&file_path, "# Existing rules\n\nSome content").unwrap();

        let result = write_template_file(&file_path, "# Agentmap Integration\n\nNew content");

        assert!(matches!(result, TemplateWriteResult::Appended));
        let content = fs::read_to_string(&file_path).unwrap();
        assert!(content.contains("# Existing rules"));
        assert!(content.contains("# Agentmap Integration"));
    }

    #[test]
    fn test_write_template_skips_if_marker_exists() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join(".cursorrules");

        fs::write(
            &file_path,
            "# Existing\n\n# Agentmap Integration\n\nAlready here",
        )
        .unwrap();

        let result = write_template_file(&file_path, "# Agentmap Integration\n\nDuplicate");

        assert!(matches!(result, TemplateWriteResult::Skipped(_)));
    }

    #[test]
    fn test_write_template_skips_alt_marker() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("CLAUDE.md");

        fs::write(
            &file_path,
            "# Project\n\n## Agentmap Documentation\n\nAlready here",
        )
        .unwrap();

        let result = write_template_file(&file_path, "## Agentmap Documentation\n\nDuplicate");

        assert!(matches!(result, TemplateWriteResult::Skipped(_)));
    }

    #[test]
    fn test_run_templates_all() {
        let temp = TempDir::new().unwrap();

        run_templates(temp.path(), None, ".agentmap").unwrap();

        assert!(temp.path().join(".cursorrules").exists());
        assert!(temp.path().join("CLAUDE.md").exists());
        assert!(temp.path().join("AGENTS.md").exists());
    }

    #[test]
    fn test_run_templates_selective() {
        let temp = TempDir::new().unwrap();

        run_templates(temp.path(), Some("cursor".to_string()), ".agentmap").unwrap();

        assert!(temp.path().join(".cursorrules").exists());
        assert!(!temp.path().join("CLAUDE.md").exists());
        assert!(!temp.path().join("AGENTS.md").exists());
    }

    #[test]
    fn test_run_templates_multiple_selective() {
        let temp = TempDir::new().unwrap();

        run_templates(temp.path(), Some("cursor,claude".to_string()), ".agentmap").unwrap();

        assert!(temp.path().join(".cursorrules").exists());
        assert!(temp.path().join("CLAUDE.md").exists());
        assert!(!temp.path().join("AGENTS.md").exists());
    }
}
