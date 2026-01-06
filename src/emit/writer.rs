//! Output writer for hierarchical content structure.
//!
//! Supports both legacy flat output and new hierarchical module-based output.

use anyhow::Result;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Legacy flat output bundle (for backward compatibility)
pub struct OutputBundle {
    pub outline: String,
    pub memory: String,
    pub agents_md: String,
    pub imports: String,
}

/// Content for a single module
#[derive(Debug, Clone, Default)]
pub struct ModuleContent {
    /// MODULE.md content
    pub module_md: String,
    /// outline.md content (symbols for large files)
    pub outline: String,
    /// memory.md content (warnings, TODOs)
    pub memory: String,
    /// imports.md content (dependencies)
    pub imports: String,
}

/// New hierarchical output bundle
#[derive(Debug, Clone)]
pub struct HierarchicalOutput {
    /// INDEX.md content (global routing table)
    pub index_md: String,
    /// Per-module content, keyed by module slug
    pub modules: HashMap<String, ModuleContent>,
    /// Optional L2 file-level docs, keyed by file slug
    pub files: HashMap<String, String>,
}

impl HierarchicalOutput {
    pub fn new(index_md: String) -> Self {
        Self {
            index_md,
            modules: HashMap::new(),
            files: HashMap::new(),
        }
    }

    /// Add content for a module
    pub fn add_module(&mut self, slug: String, content: ModuleContent) {
        self.modules.insert(slug, content);
    }

    /// Add L2 file documentation
    pub fn add_file(&mut self, slug: String, content: String) {
        self.files.insert(slug, content);
    }

    /// Count total files that would be written
    pub fn file_count(&self) -> usize {
        // INDEX.md + (up to 4 files per module) + L2 files
        1 + self
            .modules
            .values()
            .map(|m| {
                let mut count = 0;
                if !m.module_md.is_empty() {
                    count += 1;
                }
                if !m.outline.is_empty() {
                    count += 1;
                }
                if !m.memory.is_empty() {
                    count += 1;
                }
                if !m.imports.is_empty() {
                    count += 1;
                }
                count
            })
            .sum::<usize>()
            + self.files.len()
    }
}

/// Write legacy flat output (backward compatibility)
pub fn write_outputs(output_dir: &Path, bundle: &OutputBundle, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("Dry run mode - would write to: {}", output_dir.display());
        println!("  outline.md: {} bytes", bundle.outline.len());
        println!("  memory.md: {} bytes", bundle.memory.len());
        println!("  imports.md: {} bytes", bundle.imports.len());
        println!("  AGENTS.md: {} bytes", bundle.agents_md.len());
        return Ok(());
    }

    fs::create_dir_all(output_dir)?;

    fs::write(output_dir.join("outline.md"), &bundle.outline)?;
    fs::write(output_dir.join("memory.md"), &bundle.memory)?;
    fs::write(output_dir.join("imports.md"), &bundle.imports)?;
    fs::write(output_dir.join("AGENTS.md"), &bundle.agents_md)?;

    Ok(())
}

/// Write hierarchical output structure
pub fn write_hierarchical(
    output_dir: &Path,
    output: &HierarchicalOutput,
    dry_run: bool,
) -> Result<()> {
    if dry_run {
        print_hierarchical_dry_run(output_dir, output);
        return Ok(());
    }

    cleanup_legacy_files(output_dir)?;

    fs::create_dir_all(output_dir)?;

    fs::write(output_dir.join("INDEX.md"), &output.index_md)?;

    let modules_dir = output_dir.join("modules");
    if !output.modules.is_empty() {
        fs::create_dir_all(&modules_dir)?;
    }

    for (slug, content) in &output.modules {
        let module_dir = modules_dir.join(slug);
        fs::create_dir_all(&module_dir)?;

        if !content.module_md.is_empty() {
            fs::write(module_dir.join("MODULE.md"), &content.module_md)?;
        }
        if !content.outline.is_empty() {
            fs::write(module_dir.join("outline.md"), &content.outline)?;
        }
        if !content.memory.is_empty() {
            fs::write(module_dir.join("memory.md"), &content.memory)?;
        }
        if !content.imports.is_empty() {
            fs::write(module_dir.join("imports.md"), &content.imports)?;
        }
    }

    if !output.files.is_empty() {
        let files_dir = output_dir.join("files");
        fs::create_dir_all(&files_dir)?;

        for (slug, content) in &output.files {
            fs::write(files_dir.join(format!("{}.md", slug)), content)?;
        }
    }

    Ok(())
}

/// Print what would be written in dry-run mode
fn print_hierarchical_dry_run(output_dir: &Path, output: &HierarchicalOutput) {
    println!("Dry run mode - hierarchical structure:");
    println!("  {}/", output_dir.display());
    println!("  ├── INDEX.md ({} bytes)", output.index_md.len());

    if !output.modules.is_empty() {
        println!("  ├── modules/");

        let mut slugs: Vec<_> = output.modules.keys().collect();
        slugs.sort();

        for (i, slug) in slugs.iter().enumerate() {
            let content = &output.modules[*slug];
            let prefix = if i == slugs.len() - 1 && output.files.is_empty() {
                "└"
            } else {
                "├"
            };
            println!("  │   {}── {}/", prefix, slug);

            let mut files = Vec::new();
            if !content.module_md.is_empty() {
                files.push(format!("MODULE.md ({} bytes)", content.module_md.len()));
            }
            if !content.outline.is_empty() {
                files.push(format!("outline.md ({} bytes)", content.outline.len()));
            }
            if !content.memory.is_empty() {
                files.push(format!("memory.md ({} bytes)", content.memory.len()));
            }
            if !content.imports.is_empty() {
                files.push(format!("imports.md ({} bytes)", content.imports.len()));
            }

            for (j, file) in files.iter().enumerate() {
                let file_prefix = if j == files.len() - 1 { "└" } else { "├" };
                println!("  │   │   {}── {}", file_prefix, file);
            }
        }
    }

    if !output.files.is_empty() {
        println!("  └── files/");
        let mut slugs: Vec<_> = output.files.keys().collect();
        slugs.sort();

        for (i, slug) in slugs.iter().enumerate() {
            let content = &output.files[*slug];
            let prefix = if i == slugs.len() - 1 { "└" } else { "├" };
            println!("      {}── {}.md ({} bytes)", prefix, slug, content.len());
        }
    }

    println!("\nTotal: {} files", output.file_count());
}

fn cleanup_legacy_files(output_dir: &Path) -> Result<()> {
    if !output_dir.exists() {
        return Ok(());
    }

    let legacy_files = ["AGENTS.md", "outline.md", "memory.md", "imports.md"];
    for file in legacy_files {
        let path = output_dir.join(file);
        if path.exists() {
            fs::remove_file(path)?;
        }
    }

    Ok(())
}

/// Check if output directory has old flat structure
pub fn has_legacy_structure(output_dir: &Path) -> bool {
    output_dir.join("AGENTS.md").exists() && !output_dir.join("INDEX.md").exists()
}

/// Convert module slug to valid directory name
pub fn slug_to_dir_name(slug: &str) -> String {
    slug.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hierarchical_output_file_count() {
        let mut output = HierarchicalOutput::new("# INDEX".to_string());

        output.add_module(
            "src-analyze".to_string(),
            ModuleContent {
                module_md: "# Module".to_string(),
                outline: "# Outline".to_string(),
                memory: String::new(),
                imports: "# Imports".to_string(),
            },
        );

        // 1 (INDEX) + 3 (non-empty module files)
        assert_eq!(output.file_count(), 4);
    }

    #[test]
    fn test_write_hierarchical_dry_run() {
        let output = HierarchicalOutput::new("# INDEX".to_string());
        let result = write_hierarchical(Path::new("/tmp/test"), &output, true);
        assert!(result.is_ok());
    }

    #[test]
    fn test_slug_to_dir_name() {
        assert_eq!(slug_to_dir_name("src-analyze"), "src-analyze");
        assert_eq!(slug_to_dir_name("root"), "root");
    }
}
