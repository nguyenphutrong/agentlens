use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::analyze::{
    detect_modules, extract_imports, extract_memory_markers, extract_symbols, FileGraph,
};
use crate::cli::Args;
use crate::emit::{
    calculate_module_state, current_timestamp, write_hierarchical, HierarchicalOutput, Manifest,
};
use crate::generate::{
    detect_entry_points, file_path_to_slug, generate_agent_md, generate_file_doc,
    generate_index_md, generate_module_content, is_complex_file, AgentConfig, IndexConfig,
};
use crate::scan::{get_default_branch, get_diff_files, get_git_head, is_git_repo, scan_directory};
use crate::types::{FileEntry, MemoryEntry, Symbol};

pub fn run_analysis(args: &Args, work_path: &Path) -> Result<()> {
    if args.verbosity() > 0 && !args.json {
        eprintln!("Scanning: {}", work_path.display());
    }

    let diff_file_set = get_diff_file_set(args, work_path);

    let max_depth = if args.depth > 0 {
        Some(args.depth)
    } else {
        None
    };

    let files = scan_directory(work_path, args.threshold, !args.no_gitignore, max_depth)
        .context("Failed to scan directory")?;

    let files: Vec<_> = if let Some(ref diff_set) = diff_file_set {
        files
            .into_iter()
            .filter(|f| diff_set.contains(&f.relative_path))
            .collect()
    } else {
        files
    };

    if args.verbosity() > 0 && !args.json {
        eprintln!("  Files scanned: {}", files.len());
    }

    let (all_memory, all_symbols, large_file_symbols, file_graph) = analyze_files(&files)?;

    if args.verbosity() > 0 && !args.json {
        eprintln!(
            "  Large files (>{} lines): {}",
            args.threshold,
            large_file_symbols.len()
        );
        eprintln!("  Memory markers found: {}", all_memory.len());
    }

    let entry_points = detect_entry_points(&files);
    let hub_files = file_graph.hub_files();

    if args.verbosity() > 0 && !args.json {
        eprintln!("  Hub files (3+ importers): {}", hub_files.len());
    }

    let output_path = if args.output.is_absolute() {
        args.output.clone()
    } else {
        work_path.join(&args.output)
    };

    run_hierarchical_output(
        args,
        work_path,
        &output_path,
        &files,
        &all_symbols,
        &all_memory,
        &file_graph,
        &entry_points,
        &hub_files,
    )
}

fn get_diff_file_set(args: &Args, work_path: &Path) -> Option<std::collections::HashSet<String>> {
    args.diff.as_ref()?;

    if !is_git_repo(work_path) {
        eprintln!("Warning: --diff requires a git repository, ignoring flag");
        return None;
    }

    let base_ref = args
        .diff
        .clone()
        .or_else(|| get_default_branch(work_path))
        .unwrap_or_else(|| "main".to_string());

    if args.verbosity() > 0 && !args.json {
        eprintln!("  Diff mode: comparing against {}", base_ref);
    }

    get_diff_files(work_path, &base_ref).map(|stats| stats.iter().map(|s| s.path.clone()).collect())
}

type AnalysisResult = (
    Vec<MemoryEntry>,
    HashMap<String, Vec<Symbol>>,
    Vec<(FileEntry, Vec<Symbol>)>,
    FileGraph,
);

fn analyze_files(files: &[FileEntry]) -> Result<AnalysisResult> {
    let mut all_memory: Vec<MemoryEntry> = Vec::new();
    let mut all_symbols: HashMap<String, Vec<Symbol>> = HashMap::new();
    let mut large_file_symbols: Vec<(FileEntry, Vec<Symbol>)> = Vec::new();
    let mut file_graph = FileGraph::new();

    for file in files {
        let content = match fs::read_to_string(&file.path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let memory_entries = extract_memory_markers(&content, &file.relative_path);
        all_memory.extend(memory_entries);

        let imports = extract_imports(file, &content);
        file_graph.add_file(&file.relative_path, imports);

        let symbols = extract_symbols(file, &content);
        all_symbols.insert(file.relative_path.clone(), symbols.clone());

        if file.is_large {
            large_file_symbols.push((file.clone(), symbols));
        }
    }

    Ok((all_memory, all_symbols, large_file_symbols, file_graph))
}

#[allow(clippy::too_many_arguments)]
fn run_hierarchical_output(
    args: &Args,
    work_path: &Path,
    output_path: &Path,
    files: &[FileEntry],
    all_symbols: &HashMap<String, Vec<Symbol>>,
    all_memory: &[MemoryEntry],
    file_graph: &FileGraph,
    entry_points: &[String],
    hub_files: &[(String, usize)],
) -> Result<()> {
    let modules = detect_modules(files);

    if args.verbosity() > 0 {
        eprintln!("  Modules detected: {}", modules.len());
    }

    if args.verbosity() > 1 {
        for module in &modules {
            eprintln!(
                "    {} ({} files, {:?})",
                module.slug,
                module.files.len(),
                module.boundary_type
            );
        }
    }

    let mut manifest = if args.force {
        Manifest::default()
    } else {
        Manifest::load(output_path)
    };

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

    let modules_to_regenerate: Vec<_> = modules
        .iter()
        .filter(|m| {
            let state = &module_states[&m.slug];
            manifest.needs_regeneration(&m.slug, state)
        })
        .collect();

    if args.verbosity() > 0 && !args.force {
        eprintln!(
            "  Regenerating {}/{} modules",
            modules_to_regenerate.len(),
            modules.len()
        );
    }

    let hub_module_slugs: Vec<(String, usize)> = hub_files
        .iter()
        .filter_map(|(path, count)| {
            modules
                .iter()
                .find(|m| m.files.contains(path))
                .map(|m| (m.slug.clone(), *count))
        })
        .collect();

    let index_config = IndexConfig {
        modules: &modules,
        memory_entries: all_memory,
        entry_points,
        hub_modules: &hub_module_slugs,
        project_name: None,
    };
    let index_md = generate_index_md(&index_config);
    let mut output = HierarchicalOutput::new(index_md);

    let warning_count = all_memory
        .iter()
        .filter(|m| m.priority == crate::types::Priority::High)
        .count();

    let git_head = get_git_head(work_path);
    let agent_config = AgentConfig {
        modules: &modules,
        total_files: files.len(),
        warning_count,
        git_head: git_head.as_deref(),
        generated_at: current_timestamp(),
        project_name: None,
    };
    let agent_md = generate_agent_md(&agent_config);
    output.set_agent_md(agent_md);

    let large_file_symbols: Vec<(FileEntry, Vec<Symbol>)> = files
        .iter()
        .filter(|f| f.is_large)
        .filter_map(|f| {
            all_symbols
                .get(&f.relative_path)
                .map(|s| (f.clone(), s.clone()))
        })
        .collect();

    for module in &modules_to_regenerate {
        let module_memory: Vec<_> = all_memory
            .iter()
            .filter(|m| module.files.contains(&m.source_file))
            .cloned()
            .collect();

        let content = generate_module_content(
            module,
            files,
            &large_file_symbols,
            &module_memory,
            file_graph,
        );

        output.add_module(module.slug.clone(), content);

        for file_path in &module.files {
            let file = match files.iter().find(|f| &f.relative_path == file_path) {
                Some(f) => f,
                None => continue,
            };
            let symbols = all_symbols.get(file_path).map_or(&[][..], |v| v);
            if is_complex_file(file, symbols, args.complex_threshold, 50) {
                let file_memory: Vec<_> = all_memory
                    .iter()
                    .filter(|m| &m.source_file == file_path)
                    .cloned()
                    .collect();
                let file_doc = generate_file_doc(file, symbols, &file_memory, &module.slug);
                let file_slug = file_path_to_slug(&file.relative_path);
                output.add_file(file_slug, file_doc);
            }
        }
    }

    write_hierarchical(output_path, &output, args.dry_run)
        .context("Failed to write hierarchical outputs")?;

    if !args.dry_run {
        manifest.version = env!("CARGO_PKG_VERSION").to_string();
        manifest.generated_at = current_timestamp();
        for (slug, state) in module_states {
            manifest.update_module(slug, state);
        }
        let current_slugs: Vec<_> = modules.iter().map(|m| m.slug.clone()).collect();
        manifest.prune_modules(&current_slugs);
        manifest
            .save(output_path)
            .context("Failed to save manifest")?;
    }

    if args.verbosity() > 0 && !args.dry_run {
        eprintln!("\nGenerated hierarchical structure:");
        eprintln!("  {}/INDEX.md", output_path.display());
        if output.agent_md.is_some() {
            eprintln!("  {}/AGENT.md", output_path.display());
        }
        eprintln!(
            "  {}/modules/ ({} modules regenerated)",
            output_path.display(),
            output.modules.len()
        );
        if !output.files.is_empty() {
            eprintln!(
                "  {}/files/ ({} complex files)",
                output_path.display(),
                output.files.len()
            );
        }
    }

    Ok(())
}
