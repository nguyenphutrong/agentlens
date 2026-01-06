use anyhow::Result;
use std::collections::HashMap;
use std::path::Path;

use crate::analyze::detect_modules;
use crate::cli::Args;
use crate::emit::{calculate_module_state, Manifest};
use crate::scan::scan_directory;
use crate::types::FileEntry;

pub struct CheckResult {
    pub is_stale: bool,
    pub stale_modules: Vec<String>,
    pub new_modules: Vec<String>,
    pub removed_modules: Vec<String>,
}

pub fn check_staleness(args: &Args, work_path: &Path) -> Result<CheckResult> {
    let output_path = if args.output.is_absolute() {
        args.output.clone()
    } else {
        work_path.join(&args.output)
    };

    let manifest = Manifest::load(&output_path);

    let max_depth = if args.depth > 0 {
        Some(args.depth)
    } else {
        None
    };

    let files: Vec<FileEntry> =
        scan_directory(work_path, args.threshold, !args.no_gitignore, max_depth)?;

    let modules = detect_modules(&files);

    let module_states: HashMap<String, _> = modules
        .iter()
        .map(|m| {
            let module_files: Vec<_> = files
                .iter()
                .filter(|f| m.files.contains(&f.relative_path))
                .collect();
            (m.slug.clone(), calculate_module_state(&module_files))
        })
        .collect();

    let current_slugs: std::collections::HashSet<_> =
        modules.iter().map(|m| m.slug.clone()).collect();
    let manifest_slugs: std::collections::HashSet<_> = manifest.modules.keys().cloned().collect();

    let mut stale_modules = Vec::new();
    let mut new_modules = Vec::new();

    for module in &modules {
        let state = &module_states[&module.slug];
        if manifest.needs_regeneration(&module.slug, state) {
            if manifest_slugs.contains(&module.slug) {
                stale_modules.push(module.slug.clone());
            } else {
                new_modules.push(module.slug.clone());
            }
        }
    }

    let removed_modules: Vec<_> = manifest_slugs.difference(&current_slugs).cloned().collect();

    let is_stale =
        !stale_modules.is_empty() || !new_modules.is_empty() || !removed_modules.is_empty();

    Ok(CheckResult {
        is_stale,
        stale_modules,
        new_modules,
        removed_modules,
    })
}

pub fn run_check(args: &Args, work_path: &Path) -> Result<i32> {
    let result = check_staleness(args, work_path)?;

    if result.is_stale {
        eprintln!("Documentation is stale:");

        if !result.stale_modules.is_empty() {
            eprintln!("  Modified modules: {}", result.stale_modules.join(", "));
        }
        if !result.new_modules.is_empty() {
            eprintln!("  New modules: {}", result.new_modules.join(", "));
        }
        if !result.removed_modules.is_empty() {
            eprintln!("  Removed modules: {}", result.removed_modules.join(", "));
        }

        eprintln!("\nRun 'agentmap' to regenerate documentation.");
        Ok(1)
    } else {
        if args.verbosity() > 0 {
            eprintln!("Documentation is up to date.");
        }
        Ok(0)
    }
}
