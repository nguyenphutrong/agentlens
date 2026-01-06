use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::analyze::ModuleInfo;
use crate::scan::DiffStat;
use crate::types::{FileEntry, MemoryEntry, Symbol};

#[derive(Serialize)]
pub struct JsonOutput {
    pub version: String,
    pub generated_at: DateTime<Utc>,
    pub project: ProjectInfo,
    pub modules: Vec<ModuleOutput>,
    pub files: Vec<FileEntry>,
    pub large_files: Vec<LargeFileEntry>,
    pub memory: Vec<MemoryEntry>,
    pub entry_points: Vec<String>,
    pub critical_files: Vec<CriticalFile>,
    pub hub_files: Vec<HubFile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diff: Option<DiffInfo>,
}

#[derive(Serialize)]
pub struct ProjectInfo {
    pub path: String,
    pub files_scanned: usize,
    pub large_files_count: usize,
    pub memory_markers_count: usize,
    pub modules_count: usize,
}

#[derive(Serialize, Clone)]
pub struct ModuleOutput {
    pub slug: String,
    pub path: String,
    pub boundary_type: String,
    pub file_count: usize,
    pub files: Vec<String>,
    pub entry_point: Option<String>,
    pub warning_count: usize,
    pub symbol_count: usize,
    pub is_hub: bool,
}

impl ModuleOutput {
    pub fn from_module_info(
        module: &ModuleInfo,
        memory: &[MemoryEntry],
        symbols: &[(FileEntry, Vec<Symbol>)],
        hub_files: &[(String, usize)],
    ) -> Self {
        let warning_count = memory
            .iter()
            .filter(|m| module.files.contains(&m.source_file))
            .count();

        let symbol_count = symbols
            .iter()
            .filter(|(f, _)| module.files.contains(&f.relative_path))
            .map(|(_, s)| s.len())
            .sum();

        let is_hub = module
            .files
            .iter()
            .any(|f| hub_files.iter().any(|(path, _)| path == f));

        Self {
            slug: module.slug.clone(),
            path: module.path.clone(),
            boundary_type: module.boundary_type.as_str().to_string(),
            file_count: module.files.len(),
            files: module.files.clone(),
            entry_point: module.entry_point.clone(),
            warning_count,
            symbol_count,
            is_hub,
        }
    }
}

#[derive(Serialize)]
pub struct LargeFileEntry {
    pub path: String,
    pub line_count: usize,
    pub language: String,
    pub symbols: Vec<Symbol>,
}

#[derive(Serialize)]
pub struct CriticalFile {
    pub path: String,
    pub high_priority_markers: usize,
}

#[derive(Serialize)]
pub struct HubFile {
    pub path: String,
    pub imported_by: usize,
}

#[derive(Serialize)]
pub struct DiffInfo {
    pub base_ref: String,
    pub files: Vec<DiffStat>,
}

impl JsonOutput {
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|_| "{}".to_string())
    }
}
