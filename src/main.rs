use anyhow::{Context, Result};
use chrono::Utc;
use clap::Parser;
use std::fs;

use agentmap::analyze::{extract_imports, extract_memory_markers, extract_symbols, FileGraph};
use agentmap::cli::Args;
use agentmap::emit::{
    write_outputs, CriticalFile, DiffInfo, HubFile, JsonOutput, LargeFileEntry, OutputBundle,
    ProjectInfo,
};
use agentmap::generate::{
    detect_entry_points, generate_agents_md, generate_imports, generate_memory, generate_outline,
    get_critical_files, AgentsConfig,
};
use agentmap::scan::{
    cleanup_temp, clone_to_temp, get_default_branch, get_diff_files, is_git_repo, scan_directory,
    DiffStat,
};
use agentmap::types::{FileEntry, MemoryEntry, Symbol};

fn main() -> Result<()> {
    let args = Args::parse();

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

        if file.is_large {
            let symbols = extract_symbols(file, &content);
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

    let outline = generate_outline(&large_file_symbols);
    let memory = generate_memory(&all_memory);
    let imports = generate_imports(&file_graph);

    let critical_files = get_critical_files(&all_memory);
    let entry_points = detect_entry_points(&files);
    let large_files_refs: Vec<_> = large_file_symbols.iter().map(|(f, _)| f.clone()).collect();
    let hub_files = file_graph.hub_files();

    if args.verbosity() > 0 && !args.json {
        eprintln!("  Hub files (3+ importers): {}", hub_files.len());
    }

    let diff_base_ref = args
        .diff
        .clone()
        .or_else(|| get_default_branch(work_path))
        .unwrap_or_else(|| "main".to_string());

    let agents_config = AgentsConfig {
        large_files: &large_files_refs,
        critical_files: &critical_files,
        entry_points: &entry_points,
        hub_files: &hub_files,
        diff_stats: diff_stats.as_deref(),
        diff_base: if diff_stats.is_some() {
            Some(diff_base_ref.as_str())
        } else {
            None
        },
    };

    let agents_md = generate_agents_md(&agents_config);

    if args.json {
        let json_output = JsonOutput {
            version: env!("CARGO_PKG_VERSION").to_string(),
            generated_at: Utc::now(),
            project: ProjectInfo {
                path: work_path.display().to_string(),
                files_scanned: files.len(),
                large_files_count: large_file_symbols.len(),
                memory_markers_count: all_memory.len(),
            },
            files: files.clone(),
            large_files: large_file_symbols
                .iter()
                .map(|(f, syms)| LargeFileEntry {
                    path: f.relative_path.clone(),
                    line_count: f.line_count,
                    language: format!("{:?}", f.language),
                    symbols: syms.clone(),
                })
                .collect(),
            memory: all_memory.clone(),
            entry_points: entry_points.clone(),
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
            diff: diff_stats.as_ref().map(|stats| DiffInfo {
                base_ref: diff_base_ref.clone(),
                files: stats.clone(),
            }),
        };
        println!("{}", json_output.to_json());
        return Ok(());
    }

    let bundle = OutputBundle {
        outline,
        memory,
        imports,
        agents_md,
    };

    // Output path should be relative to the target project, not CWD
    let output_path = if args.output.is_absolute() {
        args.output.clone()
    } else {
        work_path.join(&args.output)
    };

    write_outputs(&output_path, &bundle, args.dry_run).context("Failed to write outputs")?;

    if args.verbosity() > 0 && !args.dry_run {
        eprintln!("\nGenerated:");
        eprintln!("  {}/outline.md", output_path.display());
        eprintln!("  {}/memory.md", output_path.display());
        eprintln!("  {}/imports.md", output_path.display());
        eprintln!("  {}/AGENTS.md", output_path.display());
    }

    Ok(())
}
