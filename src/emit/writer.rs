use anyhow::Result;
use std::path::Path;

pub struct OutputBundle {
    pub outline: String,
    pub memory: String,
    pub agents_md: String,
    pub imports: String,
}

pub fn write_outputs(output_dir: &Path, bundle: &OutputBundle, dry_run: bool) -> Result<()> {
    if dry_run {
        println!("Dry run mode - would write to: {}", output_dir.display());
        println!("  outline.md: {} bytes", bundle.outline.len());
        println!("  memory.md: {} bytes", bundle.memory.len());
        println!("  imports.md: {} bytes", bundle.imports.len());
        println!("  AGENTS.md: {} bytes", bundle.agents_md.len());
        return Ok(());
    }

    std::fs::create_dir_all(output_dir)?;

    std::fs::write(output_dir.join("outline.md"), &bundle.outline)?;
    std::fs::write(output_dir.join("memory.md"), &bundle.memory)?;
    std::fs::write(output_dir.join("imports.md"), &bundle.imports)?;
    std::fs::write(output_dir.join("AGENTS.md"), &bundle.agents_md)?;

    Ok(())
}
