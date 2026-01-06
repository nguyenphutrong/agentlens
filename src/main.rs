use anyhow::{Context, Result};
use chrono::Utc;
use clap::Parser;
use std::collections::HashMap;
use std::fs;

use agentmap::analyze::{
    detect_modules, extract_imports, extract_memory_markers, extract_symbols, FileGraph, ModuleInfo,
};
use agentmap::cli::{
    install_hooks, remove_hooks, run_update, run_watch, Args, Command, HooksAction,
};
use agentmap::emit::{
    calculate_module_state, current_timestamp, write_hierarchical, CriticalFile, DiffInfo,
    HierarchicalOutput, HubFile, JsonOutput, LargeFileEntry, Manifest, ModuleOutput, ProjectInfo,
};
use agentmap::generate::{
    detect_entry_points, file_path_to_slug, generate_file_doc, generate_index_md,
    generate_module_content, get_critical_files, is_complex_file, IndexConfig,
};
use agentmap::scan::{
    cleanup_temp, clone_to_temp, get_default_branch, get_diff_files, is_git_repo, scan_directory,
    DiffStat,
};
use agentmap::types::{FileEntry, MemoryEntry, Symbol};
use agentmap::Config;

fn main() -> Result<()> {
    let args = Args::parse();

    match args.command.clone() {
        Some(Command::Update) => return run_update(),
        Some(Command::Watch { debounce }) => {
            let args = args.with_config();
            return run_watch(&args, debounce);
        }
        Some(Command::Hooks { action }) => {
            let path = args.path.canonicalize().unwrap_or(args.path.clone());
            return match action {
                HooksAction::Install => install_hooks(&path),
                HooksAction::Remove => remove_hooks(&path),
            };
        }
        Some(Command::Init { config, hooks }) => {
            let path = args.path.canonicalize().unwrap_or(args.path.clone());
            return run_init(&path, config, hooks);
        }
        None => {}
    }

    let args = args.with_config();

    args.validate()
        .map_err(|e| anyhow::anyhow!(e))
        .context("Invalid arguments")?;

    let (work_path, temp_dir) = if args.is_remote() {
        let url = args.path.to_string_lossy().to_string();
        if args.verbosity() > 0 && !args.json {
            eprintln!("Cloning remote repository: {}", url);
        }
        let temp = clone_to_temp(&url).context("Failed to clone remote repository")?;
        (temp.clone(), Some(temp))
    } else {
        (args.path.clone(), None)
    };

    let result = run_analysis(&args, &work_path);

    if let Some(ref temp) = temp_dir {
        cleanup_temp(temp);
    }

    result
}

fn run_analysis(args: &Args, work_path: &std::path::Path) -> Result<()> {
    if args.verbosity() > 0 && !args.json {
        eprintln!("Scanning: {}", work_path.display());
    }

    let diff_stats: Option<Vec<DiffStat>> = if args.diff.is_some() {
        if !is_git_repo(work_path) {
            eprintln!("Warning: --diff requires a git repository, ignoring flag");
            None
        } else {
            let base_ref_owned = args
                .diff
                .clone()
                .or_else(|| get_default_branch(work_path))
                .unwrap_or_else(|| "main".to_string());

            if args.verbosity() > 0 && !args.json {
                eprintln!("  Diff mode: comparing against {}", base_ref_owned);
            }
            get_diff_files(work_path, &base_ref_owned)
        }
    } else {
        None
    };

    let diff_file_set: Option<std::collections::HashSet<String>> = diff_stats
        .as_ref()
        .map(|stats| stats.iter().map(|s| s.path.clone()).collect());

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

    let mut all_memory: Vec<MemoryEntry> = Vec::new();
    let mut all_symbols: HashMap<String, Vec<Symbol>> = HashMap::new();
    let mut large_file_symbols: Vec<(FileEntry, Vec<Symbol>)> = Vec::new();
    let mut file_graph = FileGraph::new();

    for file in &files {
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

    let diff_base_ref = args
        .diff
        .clone()
        .or_else(|| get_default_branch(work_path))
        .unwrap_or_else(|| "main".to_string());

    let modules = detect_modules(&files);

    if args.json {
        return run_json_output(
            work_path,
            &files,
            &modules,
            &large_file_symbols,
            &all_memory,
            &entry_points,
            &hub_files,
            diff_stats.as_ref(),
            &diff_base_ref,
        );
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

#[allow(clippy::too_many_arguments)]
fn run_json_output(
    work_path: &std::path::Path,
    files: &[FileEntry],
    modules: &[ModuleInfo],
    large_file_symbols: &[(FileEntry, Vec<Symbol>)],
    all_memory: &[MemoryEntry],
    entry_points: &[String],
    hub_files: &[(String, usize)],
    diff_stats: Option<&Vec<DiffStat>>,
    diff_base_ref: &str,
) -> Result<()> {
    let critical_files = get_critical_files(all_memory);
    let module_outputs: Vec<ModuleOutput> = modules
        .iter()
        .map(|m| ModuleOutput::from_module_info(m, all_memory, large_file_symbols, hub_files))
        .collect();

    let json_output = JsonOutput {
        version: env!("CARGO_PKG_VERSION").to_string(),
        generated_at: Utc::now(),
        project: ProjectInfo {
            path: work_path.display().to_string(),
            files_scanned: files.len(),
            large_files_count: large_file_symbols.len(),
            memory_markers_count: all_memory.len(),
            modules_count: modules.len(),
        },
        modules: module_outputs,
        files: files.to_vec(),
        large_files: large_file_symbols
            .iter()
            .map(|(f, syms)| LargeFileEntry {
                path: f.relative_path.clone(),
                line_count: f.line_count,
                language: format!("{:?}", f.language),
                symbols: syms.clone(),
            })
            .collect(),
        memory: all_memory.to_vec(),
        entry_points: entry_points.to_vec(),
        critical_files: critical_files
            .iter()
            .map(|(path, count)| CriticalFile {
                path: path.clone(),
                high_priority_markers: *count,
            })
            .collect(),
        hub_files: hub_files
            .iter()
            .map(|(path, count)| HubFile {
                path: path.clone(),
                imported_by: *count,
            })
            .collect(),
        diff: diff_stats.map(|stats| DiffInfo {
            base_ref: diff_base_ref.to_string(),
            files: stats.clone(),
        }),
    };
    println!("{}", json_output.to_json());
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn run_hierarchical_output(
    args: &Args,
    _work_path: &std::path::Path,
    output_path: &std::path::Path,
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

fn run_init(path: &std::path::Path, config: bool, hooks: bool) -> Result<()> {
    let mut did_something = false;

    if config {
        match Config::create_default_file(path) {
            Ok(config_path) => {
                eprintln!("Created: {}", config_path.display());
                did_something = true;
            }
            Err(e) => eprintln!("Failed to create config: {}", e),
        }
    }

    if hooks {
        install_hooks(path)?;
        did_something = true;
    }

    if !did_something {
        eprintln!("Usage: agentmap init [--config] [--hooks]");
        eprintln!("  --config  Create agentmap.toml with default settings");
        eprintln!("  --hooks   Install git hooks for automatic regeneration");
    }

    Ok(())
}
