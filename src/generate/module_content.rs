//! Module-level content generators for hierarchical output.
//!
//! Generates per-module documentation files:
//! - MODULE.md: Module summary and navigation
//! - outline.md: Symbol maps for large files in this module
//! - memory.md: Warnings/TODOs for this module
//! - imports.md: Dependencies within/from this module

use crate::analyze::{FileGraph, ModuleInfo};
use crate::emit::ModuleContent;
use crate::types::{FileEntry, MemoryEntry, Priority, Symbol};

/// Generate all content for a single module
pub fn generate_module_content(
    module: &ModuleInfo,
    files: &[FileEntry],
    symbols: &[(FileEntry, Vec<Symbol>)],
    memory: &[MemoryEntry],
    graph: &FileGraph,
) -> ModuleContent {
    let module_files: Vec<&FileEntry> = files
        .iter()
        .filter(|f| module.files.contains(&f.relative_path))
        .collect();

    let outline = generate_module_outline(module, symbols);
    let memory_content = generate_module_memory(module, memory);
    let imports = generate_module_imports(module, graph);

    let has_outline = !outline.is_empty();
    let has_memory = !memory_content.is_empty();
    let has_imports = !imports.is_empty();

    ModuleContent {
        module_md: generate_module_md(module, &module_files, has_outline, has_memory, has_imports),
        outline,
        memory: memory_content,
        imports,
    }
}

/// Generate MODULE.md content
fn generate_module_md(
    module: &ModuleInfo,
    files: &[&FileEntry],
    has_outline: bool,
    has_memory: bool,
    has_imports: bool,
) -> String {
    let mut output = String::new();

    // Header
    let title = if module.path.is_empty() || module.slug == "root" {
        "Root Module".to_string()
    } else {
        format!("Module: {}", module.path)
    };
    output.push_str(&format!("# {}\n\n", title));

    // Navigation
    output.push_str("[← Back to INDEX](../../INDEX.md)\n\n");

    // Module info
    output.push_str(&format!(
        "**Type:** {} | **Files:** {}\n\n",
        module.boundary_type.as_str(),
        module.file_count()
    ));

    // Entry point
    if let Some(ref entry) = module.entry_point {
        output.push_str(&format!("**Entry point:** `{}`\n\n", entry));
    }

    // Files in this module
    if !files.is_empty() {
        output.push_str("## Files\n\n");
        output.push_str("| File | Lines | Large |\n");
        output.push_str("| ---- | ----- | ----- |\n");

        for file in files {
            let large_indicator = if file.is_large { "📊" } else { "" };
            output.push_str(&format!(
                "| `{}` | {} | {} |\n",
                file.relative_path, file.line_count, large_indicator
            ));
        }
        output.push('\n');
    }

    // Child modules
    if !module.children.is_empty() {
        output.push_str("## Child Modules\n\n");
        for child in &module.children {
            output.push_str(&format!("- [{}](../{}/MODULE.md)\n", child, child));
        }
        output.push('\n');
    }

    if has_outline || has_memory || has_imports {
        output.push_str("## Documentation\n\n");
        if has_outline {
            output.push_str("- [outline.md](outline.md) - Symbol maps for large files\n");
        }
        if has_memory {
            output.push_str("- [memory.md](memory.md) - Warnings and TODOs\n");
        }
        if has_imports {
            output.push_str("- [imports.md](imports.md) - Dependencies\n");
        }
    }

    output
}

/// Generate module-scoped outline.md
/// Returns empty string if no large files with symbols exist (skips file creation)
fn generate_module_outline(module: &ModuleInfo, symbols: &[(FileEntry, Vec<Symbol>)]) -> String {
    // Filter to only files in this module
    let module_symbols: Vec<_> = symbols
        .iter()
        .filter(|(f, _)| module.files.contains(&f.relative_path))
        .collect();

    if module_symbols.is_empty() {
        return String::new();
    }

    let mut output = String::new();

    output.push_str("# Outline\n\n");
    output.push_str("[← Back to MODULE](MODULE.md) | [← Back to INDEX](../../INDEX.md)\n\n");

    output.push_str(&format!(
        "Symbol maps for {} large files in this module.\n\n",
        module_symbols.len()
    ));

    for (file, syms) in module_symbols {
        output.push_str(&format!(
            "## {} ({} lines)\n\n",
            file.relative_path, file.line_count
        ));

        if syms.is_empty() {
            output.push_str("_No symbols extracted._\n\n");
            continue;
        }

        output.push_str("| Line | Kind | Name | Visibility |\n");
        output.push_str("| ---- | ---- | ---- | ---------- |\n");

        for sym in syms {
            output.push_str(&format!(
                "| {} | {} | {} | {} |\n",
                sym.line_range.start, sym.kind, sym.name, sym.visibility
            ));
        }
        output.push('\n');
    }

    output
}

/// Generate module-scoped memory.md
/// Returns empty string if no memory markers exist (skips file creation)
fn generate_module_memory(module: &ModuleInfo, memory: &[MemoryEntry]) -> String {
    // Filter to only entries in this module
    let module_memory: Vec<_> = memory
        .iter()
        .filter(|e| module.files.contains(&e.source_file))
        .collect();

    if module_memory.is_empty() {
        return String::new();
    }

    let mut output = String::new();

    output.push_str("# Memory\n\n");
    output.push_str("[← Back to MODULE](MODULE.md) | [← Back to INDEX](../../INDEX.md)\n\n");

    // Count by priority
    let high = module_memory
        .iter()
        .filter(|e| e.priority == Priority::High)
        .count();
    let med = module_memory
        .iter()
        .filter(|e| e.priority == Priority::Medium)
        .count();
    let low = module_memory
        .iter()
        .filter(|e| e.priority == Priority::Low)
        .count();

    output.push_str("## Summary\n\n");
    output.push_str(&format!(
        "| High 🔴 | Medium 🟡 | Low 🟢 |\n| {} | {} | {} |\n\n",
        high, med, low
    ));

    // Group by priority
    if high > 0 {
        output.push_str("## 🔴 High Priority\n\n");
        for entry in module_memory
            .iter()
            .filter(|e| e.priority == Priority::High)
        {
            output.push_str(&format!(
                "### `{}` ({}:{})\n\n> {}\n\n",
                entry.kind, entry.source_file, entry.line_number, entry.content
            ));
        }
    }

    if med > 0 {
        output.push_str("## 🟡 Medium Priority\n\n");
        for entry in module_memory
            .iter()
            .filter(|e| e.priority == Priority::Medium)
        {
            output.push_str(&format!(
                "### `{}` ({}:{})\n\n> {}\n\n",
                entry.kind, entry.source_file, entry.line_number, entry.content
            ));
        }
    }

    if low > 0 {
        output.push_str("## 🟢 Low Priority\n\n");
        for entry in module_memory.iter().filter(|e| e.priority == Priority::Low) {
            output.push_str(&format!(
                "### `{}` ({}:{})\n\n> {}\n\n",
                entry.kind, entry.source_file, entry.line_number, entry.content
            ));
        }
    }

    output
}

/// Generate module-scoped imports.md
fn generate_module_imports(module: &ModuleInfo, graph: &FileGraph) -> String {
    let mut output = String::new();

    output.push_str("# Imports\n\n");
    output.push_str("[← Back to MODULE](MODULE.md) | [← Back to INDEX](../../INDEX.md)\n\n");

    let mut internal_deps: Vec<String> = Vec::new();
    let mut external_deps: Vec<String> = Vec::new();
    let mut consumers: Vec<String> = Vec::new();

    for file in &module.files {
        // Get what this file imports
        if let Some(imports) = graph.imports.get(file) {
            for imp in imports {
                if module.files.iter().any(|f| f.contains(imp)) {
                    if !internal_deps.contains(imp) {
                        internal_deps.push(imp.clone());
                    }
                } else if !external_deps.contains(imp) {
                    external_deps.push(imp.clone());
                }
            }
        }

        // Get what imports this file
        if let Some(importers) = graph.importers.get(file) {
            for importer in importers {
                if !module.files.contains(importer) && !consumers.contains(importer) {
                    consumers.push(importer.clone());
                }
            }
        }
    }

    internal_deps.sort();
    external_deps.sort();
    consumers.sort();

    if internal_deps.is_empty() && external_deps.is_empty() && consumers.is_empty() {
        output.push_str("_No import relationships detected._\n");
        return output;
    }

    if !internal_deps.is_empty() {
        output.push_str("## Internal Dependencies\n\n");
        output.push_str("Dependencies within this module:\n\n");
        for dep in &internal_deps {
            output.push_str(&format!("- `{}`\n", dep));
        }
        output.push('\n');
    }

    if !external_deps.is_empty() {
        output.push_str("## External Dependencies\n\n");
        output.push_str("Dependencies from other modules:\n\n");
        for dep in &external_deps {
            output.push_str(&format!("- `{}`\n", dep));
        }
        output.push('\n');
    }

    if !consumers.is_empty() {
        output.push_str("## Consumers\n\n");
        output.push_str("Files from other modules that import from this module:\n\n");
        for consumer in &consumers {
            output.push_str(&format!("- `{}`\n", consumer));
        }
        output.push('\n');
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::analyze::BoundaryType;
    use crate::types::{Language, MemoryKind};
    use std::path::PathBuf;

    fn make_module(path: &str, files: Vec<String>) -> ModuleInfo {
        let mut module = ModuleInfo::new(path, BoundaryType::RustModule, None);
        for f in files {
            module.add_file(f);
        }
        module
    }

    fn make_file(path: &str, lines: usize) -> FileEntry {
        FileEntry {
            path: PathBuf::from(path),
            relative_path: path.to_string(),
            extension: Some("rs".to_string()),
            language: Language::Rust,
            size_bytes: 1000,
            line_count: lines,
            is_large: lines > 500,
        }
    }

    #[test]
    fn test_generate_module_md() {
        let module = make_module("src/analyze", vec!["src/analyze/mod.rs".to_string()]);
        let files = vec![make_file("src/analyze/mod.rs", 100)];
        let file_refs: Vec<&FileEntry> = files.iter().collect();

        let result = generate_module_md(&module, &file_refs, true, true, true);

        assert!(result.contains("# Module: src/analyze"));
        assert!(result.contains("Back to INDEX"));
        assert!(result.contains("src/analyze/mod.rs"));
        assert!(result.contains("outline.md"));
        assert!(result.contains("memory.md"));
        assert!(result.contains("imports.md"));
    }

    #[test]
    fn test_generate_module_md_no_docs() {
        let module = make_module("src/analyze", vec!["src/analyze/mod.rs".to_string()]);
        let files = vec![make_file("src/analyze/mod.rs", 100)];
        let file_refs: Vec<&FileEntry> = files.iter().collect();

        let result = generate_module_md(&module, &file_refs, false, false, false);

        assert!(result.contains("# Module: src/analyze"));
        assert!(!result.contains("outline.md"));
        assert!(!result.contains("memory.md"));
        assert!(!result.contains("imports.md"));
    }

    #[test]
    fn test_generate_module_outline_empty() {
        let module = make_module("src/small", vec!["src/small/mod.rs".to_string()]);
        let symbols: Vec<(FileEntry, Vec<Symbol>)> = vec![];

        let result = generate_module_outline(&module, &symbols);

        assert!(result.is_empty());
    }

    #[test]
    fn test_generate_module_memory_empty() {
        let module = make_module("src/clean", vec!["src/clean/mod.rs".to_string()]);
        let memory: Vec<MemoryEntry> = vec![];

        let result = generate_module_memory(&module, &memory);

        assert!(result.is_empty());
    }

    #[test]
    fn test_generate_module_memory_with_entries() {
        let module = make_module("src/warn", vec!["src/warn/mod.rs".to_string()]);
        let memory = vec![MemoryEntry {
            kind: MemoryKind::Warning,
            content: "This is dangerous".to_string(),
            source_file: "src/warn/mod.rs".to_string(),
            line_number: 10,
            priority: Priority::High,
        }];

        let result = generate_module_memory(&module, &memory);

        assert!(result.contains("High Priority"));
        assert!(result.contains("This is dangerous"));
    }

    #[test]
    fn test_generate_module_imports_empty() {
        let module = make_module("src/isolated", vec!["src/isolated/mod.rs".to_string()]);
        let graph = FileGraph::new();

        let result = generate_module_imports(&module, &graph);

        assert!(result.contains("No import relationships"));
    }
}
