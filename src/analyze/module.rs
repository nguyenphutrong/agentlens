//! Module boundary detection for hierarchical content architecture.
//!
//! Detects semantic module boundaries based on:
//! - Explicit markers: mod.rs, __init__.py, index.{js,ts,tsx,jsx}
//! - Implicit boundaries: directories with 5+ source files

use crate::types::FileEntry;
use serde::Serialize;
use std::collections::{HashMap, HashSet};
use std::path::Path;

/// Minimum number of files for implicit module detection
const IMPLICIT_MODULE_THRESHOLD: usize = 5;

/// Information about a detected module
#[derive(Debug, Clone, Serialize)]
pub struct ModuleInfo {
    /// URL-safe slug: "src/analyze" → "src-analyze"
    pub slug: String,
    /// Original path relative to project root
    pub path: String,
    /// Files belonging to this module (relative paths)
    pub files: Vec<String>,
    /// Entry point file if detected (e.g., mod.rs, index.ts)
    pub entry_point: Option<String>,
    /// How this module was detected
    pub boundary_type: BoundaryType,
    /// Parent module slug if this is a nested module
    pub parent: Option<String>,
    /// Child module slugs
    pub children: Vec<String>,
}

/// How a module boundary was detected
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum BoundaryType {
    /// Rust: mod.rs or lib.rs
    RustModule,
    /// Python: __init__.py
    PythonPackage,
    /// JavaScript/TypeScript: index.{js,ts,tsx,jsx}
    JsModule,
    /// Go: directory with .go files (package)
    GoPackage,
    /// Implicit: directory with 5+ source files
    Implicit,
    /// Root module (project root)
    Root,
}

impl BoundaryType {
    pub fn as_str(&self) -> &'static str {
        match self {
            BoundaryType::RustModule => "rust",
            BoundaryType::PythonPackage => "python",
            BoundaryType::JsModule => "js/ts",
            BoundaryType::GoPackage => "go",
            BoundaryType::Implicit => "implicit",
            BoundaryType::Root => "root",
        }
    }
}

impl ModuleInfo {
    /// Create a new module
    pub fn new(path: &str, boundary_type: BoundaryType, entry_point: Option<String>) -> Self {
        Self {
            slug: path_to_slug(path),
            path: path.to_string(),
            files: Vec::new(),
            entry_point,
            boundary_type,
            parent: None,
            children: Vec::new(),
        }
    }

    /// Add a file to this module
    pub fn add_file(&mut self, relative_path: String) {
        if !self.files.contains(&relative_path) {
            self.files.push(relative_path);
        }
    }

    /// Number of files in this module
    pub fn file_count(&self) -> usize {
        self.files.len()
    }
}

/// Convert a path to a URL-safe slug
/// "src/analyze" → "src-analyze"
/// "src/analyze/lang" → "src-analyze-lang"
pub fn path_to_slug(path: &str) -> String {
    if path.is_empty() || path == "." {
        return "root".to_string();
    }
    path.replace(['/', '\\'], "-")
}

/// Detect all modules in a codebase
pub fn detect_modules(files: &[FileEntry]) -> Vec<ModuleInfo> {
    let mut modules: HashMap<String, ModuleInfo> = HashMap::new();
    let mut file_to_module: HashMap<String, String> = HashMap::new();

    // First pass: detect explicit module boundaries
    for file in files {
        if let Some((module_path, boundary_type)) = detect_explicit_boundary(file) {
            let entry_point = Some(file.relative_path.clone());
            let module = ModuleInfo::new(&module_path, boundary_type, entry_point);
            modules.insert(module_path.clone(), module);
        }
    }

    // Second pass: assign files to their nearest module
    for file in files {
        let dir = get_parent_dir(&file.relative_path);
        let module_path = find_owning_module(&dir, &modules);
        file_to_module.insert(file.relative_path.clone(), module_path);
    }

    // Third pass: detect implicit modules for directories with 5+ files
    let mut dir_file_counts: HashMap<String, Vec<String>> = HashMap::new();
    for file in files {
        let dir = get_parent_dir(&file.relative_path);
        dir_file_counts
            .entry(dir)
            .or_default()
            .push(file.relative_path.clone());
    }

    for (dir, dir_files) in &dir_file_counts {
        // Skip if already an explicit module or if too few files
        if modules.contains_key(dir) || dir_files.len() < IMPLICIT_MODULE_THRESHOLD {
            continue;
        }

        // Skip if this directory is already covered by an explicit child module
        let has_explicit_child = modules.keys().any(|m| m.starts_with(dir) && m != dir);
        if has_explicit_child {
            continue;
        }

        // Create implicit module
        let module = ModuleInfo::new(dir, BoundaryType::Implicit, None);
        modules.insert(dir.clone(), module);

        // Reassign files to this new module
        for file_path in dir_files {
            file_to_module.insert(file_path.clone(), dir.clone());
        }
    }

    // Fourth pass: add files to their modules
    for (file_path, module_path) in &file_to_module {
        if let Some(module) = modules.get_mut(module_path) {
            module.add_file(file_path.clone());
        }
    }

    // Create root module for any orphaned files
    let orphan_files: Vec<String> = files
        .iter()
        .filter(|f| {
            let module_path = file_to_module.get(&f.relative_path);
            module_path.is_none() || !modules.contains_key(module_path.unwrap())
        })
        .map(|f| f.relative_path.clone())
        .collect();

    if !orphan_files.is_empty() {
        let mut root_module = ModuleInfo::new("", BoundaryType::Root, None);
        for file_path in orphan_files {
            root_module.add_file(file_path);
        }
        modules.insert("".to_string(), root_module);
    }

    // Fifth pass: establish parent-child relationships
    let module_paths: Vec<String> = modules.keys().cloned().collect();
    for path in &module_paths {
        if path.is_empty() {
            continue;
        }

        // Find parent
        let parent_path = find_parent_module(path, &module_paths);
        if let Some(parent) = parent_path {
            if let Some(module) = modules.get_mut(path) {
                module.parent = Some(path_to_slug(&parent));
            }
            if let Some(parent_module) = modules.get_mut(&parent) {
                parent_module.children.push(path_to_slug(path));
            }
        }
    }

    // Sort children alphabetically
    for module in modules.values_mut() {
        module.children.sort();
        module.files.sort();
    }

    // Convert to sorted vec
    let mut result: Vec<ModuleInfo> = modules.into_values().collect();
    result.sort_by(|a, b| a.path.cmp(&b.path));

    result
}

/// Detect if a file represents an explicit module boundary
fn detect_explicit_boundary(file: &FileEntry) -> Option<(String, BoundaryType)> {
    let filename = Path::new(&file.relative_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    let dir = get_parent_dir(&file.relative_path);

    match filename {
        // Rust: mod.rs marks a module, lib.rs marks crate root
        "mod.rs" => Some((dir, BoundaryType::RustModule)),
        "lib.rs" => {
            // lib.rs at src/ level is crate root
            if dir == "src" || dir.is_empty() {
                Some((dir, BoundaryType::RustModule))
            } else {
                None
            }
        }

        // Python: __init__.py marks a package
        "__init__.py" => Some((dir, BoundaryType::PythonPackage)),

        // JavaScript/TypeScript: index.{js,ts,tsx,jsx} marks a module
        "index.js" | "index.jsx" | "index.ts" | "index.tsx" | "index.mjs" | "index.mts" => {
            Some((dir, BoundaryType::JsModule))
        }

        _ => None,
    }
}

/// Get the parent directory of a file path
fn get_parent_dir(path: &str) -> String {
    Path::new(path)
        .parent()
        .and_then(|p| p.to_str())
        .unwrap_or("")
        .to_string()
}

/// Find which module owns a directory (walks up the tree)
fn find_owning_module(dir: &str, modules: &HashMap<String, ModuleInfo>) -> String {
    let mut current = dir.to_string();

    loop {
        if modules.contains_key(&current) {
            return current;
        }

        match Path::new(&current).parent() {
            Some(parent) => {
                let parent_str = parent.to_str().unwrap_or("");
                if parent_str.is_empty() || parent_str == current {
                    break;
                }
                current = parent_str.to_string();
            }
            None => break,
        }
    }

    // No module found, return root
    String::new()
}

/// Find the parent module of a given module path
fn find_parent_module(path: &str, all_paths: &[String]) -> Option<String> {
    let mut current = path.to_string();

    loop {
        match Path::new(&current).parent() {
            Some(parent) => {
                let parent_str = parent.to_str().unwrap_or("").to_string();
                if parent_str.is_empty() {
                    return None;
                }
                if all_paths.contains(&parent_str) {
                    return Some(parent_str);
                }
                current = parent_str;
            }
            None => return None,
        }
    }
}

/// Get files that belong to a specific module (not including child modules)
pub fn get_module_files<'a>(
    module: &ModuleInfo,
    all_modules: &[ModuleInfo],
    files: &'a [FileEntry],
) -> Vec<&'a FileEntry> {
    let child_prefixes: HashSet<&str> = all_modules
        .iter()
        .filter(|m| m.parent.as_ref() == Some(&module.slug))
        .map(|m| m.path.as_str())
        .collect();

    files
        .iter()
        .filter(|f| {
            // File must be in this module's path
            let file_dir = get_parent_dir(&f.relative_path);
            if !file_dir.starts_with(&module.path) && file_dir != module.path {
                return false;
            }

            // File must not be in a child module
            !child_prefixes
                .iter()
                .any(|prefix| file_dir.starts_with(prefix))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Language;
    use std::path::PathBuf;

    fn make_file(relative_path: &str) -> FileEntry {
        FileEntry {
            path: PathBuf::from(relative_path),
            relative_path: relative_path.to_string(),
            extension: relative_path.split('.').last().map(|s| s.to_string()),
            language: Language::Rust,
            size_bytes: 100,
            line_count: 50,
            is_large: false,
        }
    }

    #[test]
    fn test_path_to_slug() {
        assert_eq!(path_to_slug("src/analyze"), "src-analyze");
        assert_eq!(path_to_slug("src/analyze/lang"), "src-analyze-lang");
        assert_eq!(path_to_slug(""), "root");
        assert_eq!(path_to_slug("."), "root");
    }

    #[test]
    fn test_detect_rust_modules() {
        let files = vec![
            make_file("src/lib.rs"),
            make_file("src/main.rs"),
            make_file("src/analyze/mod.rs"),
            make_file("src/analyze/parser.rs"),
            make_file("src/analyze/lang/mod.rs"),
            make_file("src/analyze/lang/rust.rs"),
        ];

        let modules = detect_modules(&files);

        // Should detect: src (from lib.rs), src/analyze (from mod.rs), src/analyze/lang (from mod.rs)
        assert!(modules.iter().any(|m| m.path == "src"));
        assert!(modules.iter().any(|m| m.path == "src/analyze"));
        assert!(modules.iter().any(|m| m.path == "src/analyze/lang"));

        // Check boundary types
        let src_module = modules.iter().find(|m| m.path == "src").unwrap();
        assert_eq!(src_module.boundary_type, BoundaryType::RustModule);

        let analyze_module = modules.iter().find(|m| m.path == "src/analyze").unwrap();
        assert_eq!(analyze_module.boundary_type, BoundaryType::RustModule);
    }

    #[test]
    fn test_detect_python_packages() {
        let files = vec![
            make_file("mypackage/__init__.py"),
            make_file("mypackage/core.py"),
            make_file("mypackage/utils/__init__.py"),
            make_file("mypackage/utils/helpers.py"),
        ];

        // Override language for Python files
        let files: Vec<FileEntry> = files
            .into_iter()
            .map(|mut f| {
                f.language = Language::Python;
                f
            })
            .collect();

        let modules = detect_modules(&files);

        assert!(modules.iter().any(|m| m.path == "mypackage"));
        assert!(modules.iter().any(|m| m.path == "mypackage/utils"));

        let pkg_module = modules.iter().find(|m| m.path == "mypackage").unwrap();
        assert_eq!(pkg_module.boundary_type, BoundaryType::PythonPackage);
    }

    #[test]
    fn test_detect_js_modules() {
        let files = vec![
            make_file("src/index.ts"),
            make_file("src/app.ts"),
            make_file("src/components/index.tsx"),
            make_file("src/components/Button.tsx"),
            make_file("src/utils/index.js"),
            make_file("src/utils/format.js"),
        ];

        let modules = detect_modules(&files);

        assert!(modules.iter().any(|m| m.path == "src"));
        assert!(modules.iter().any(|m| m.path == "src/components"));
        assert!(modules.iter().any(|m| m.path == "src/utils"));

        let components = modules.iter().find(|m| m.path == "src/components").unwrap();
        assert_eq!(components.boundary_type, BoundaryType::JsModule);
    }

    #[test]
    fn test_detect_implicit_modules() {
        // Create 6 files in a directory without explicit boundary
        let files: Vec<FileEntry> = (0..6)
            .map(|i| make_file(&format!("src/services/service{}.rs", i)))
            .collect();

        let modules = detect_modules(&files);

        let services = modules.iter().find(|m| m.path == "src/services");
        assert!(services.is_some());
        assert_eq!(services.unwrap().boundary_type, BoundaryType::Implicit);
    }

    #[test]
    fn test_implicit_threshold_not_met() {
        // Only 3 files - should not create implicit module
        let files: Vec<FileEntry> = (0..3)
            .map(|i| make_file(&format!("src/small/file{}.rs", i)))
            .collect();

        let modules = detect_modules(&files);

        let small = modules.iter().find(|m| m.path == "src/small");
        assert!(small.is_none());
    }

    #[test]
    fn test_nested_module_relationships() {
        let files = vec![
            make_file("src/lib.rs"),
            make_file("src/analyze/mod.rs"),
            make_file("src/analyze/lang/mod.rs"),
        ];

        let modules = detect_modules(&files);

        let lang_module = modules
            .iter()
            .find(|m| m.path == "src/analyze/lang")
            .unwrap();
        assert_eq!(lang_module.parent, Some("src-analyze".to_string()));

        let analyze_module = modules.iter().find(|m| m.path == "src/analyze").unwrap();
        assert!(analyze_module
            .children
            .contains(&"src-analyze-lang".to_string()));
    }

    #[test]
    fn test_file_assignment() {
        let files = vec![
            make_file("src/lib.rs"),
            make_file("src/main.rs"),
            make_file("src/analyze/mod.rs"),
            make_file("src/analyze/parser.rs"),
        ];

        let modules = detect_modules(&files);

        let analyze = modules.iter().find(|m| m.path == "src/analyze").unwrap();
        assert!(analyze.files.contains(&"src/analyze/mod.rs".to_string()));
        assert!(analyze.files.contains(&"src/analyze/parser.rs".to_string()));

        let src = modules.iter().find(|m| m.path == "src").unwrap();
        assert!(src.files.contains(&"src/lib.rs".to_string()));
        assert!(src.files.contains(&"src/main.rs".to_string()));
    }
}
