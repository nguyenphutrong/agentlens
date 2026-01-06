//! Manifest tracking for incremental regeneration.
//!
//! Stores module timestamps to detect changes between runs.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::SystemTime;

const MANIFEST_FILE: &str = ".manifest.json";

/// Manifest tracking module state for incremental builds
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Manifest {
    /// Version of agentmap that generated this manifest
    pub version: String,
    /// Timestamp of last full generation
    pub generated_at: u64,
    /// Per-module state
    pub modules: HashMap<String, ModuleState>,
}

/// State for a single module
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ModuleState {
    /// Latest modification time of any file in the module (unix timestamp)
    pub latest_mtime: u64,
    /// Number of files in the module
    pub file_count: usize,
    /// Hash of file paths (to detect file additions/removals)
    pub files_hash: u64,
}

impl Manifest {
    /// Load manifest from output directory, or return empty if not found
    pub fn load(output_dir: &Path) -> Self {
        let manifest_path = output_dir.join(MANIFEST_FILE);
        if !manifest_path.exists() {
            return Self::default();
        }

        match fs::read_to_string(&manifest_path) {
            Ok(content) => serde_json::from_str(&content).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save manifest to output directory
    pub fn save(&self, output_dir: &Path) -> Result<()> {
        let manifest_path = output_dir.join(MANIFEST_FILE);
        let content = serde_json::to_string_pretty(self)?;
        fs::write(manifest_path, content)?;
        Ok(())
    }

    /// Check if a module needs regeneration
    pub fn needs_regeneration(&self, slug: &str, current_state: &ModuleState) -> bool {
        match self.modules.get(slug) {
            None => true,
            Some(old_state) => {
                old_state.latest_mtime != current_state.latest_mtime
                    || old_state.file_count != current_state.file_count
                    || old_state.files_hash != current_state.files_hash
            }
        }
    }

    /// Update module state in manifest
    pub fn update_module(&mut self, slug: String, state: ModuleState) {
        self.modules.insert(slug, state);
    }

    /// Remove modules that no longer exist
    pub fn prune_modules(&mut self, current_slugs: &[String]) {
        let current_set: std::collections::HashSet<_> = current_slugs.iter().collect();
        self.modules.retain(|slug, _| current_set.contains(slug));
    }
}

/// Calculate module state from file entries
pub fn calculate_module_state(files: &[&crate::types::FileEntry]) -> ModuleState {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut latest_mtime: u64 = 0;
    let mut hasher = DefaultHasher::new();

    for file in files {
        if let Ok(metadata) = fs::metadata(&file.path) {
            if let Ok(mtime) = metadata.modified() {
                if let Ok(duration) = mtime.duration_since(SystemTime::UNIX_EPOCH) {
                    let timestamp = duration.as_secs();
                    if timestamp > latest_mtime {
                        latest_mtime = timestamp;
                    }
                }
            }
        }

        file.relative_path.hash(&mut hasher);
    }

    ModuleState {
        latest_mtime,
        file_count: files.len(),
        files_hash: hasher.finish(),
    }
}

/// Get current unix timestamp
pub fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_needs_regeneration_new_module() {
        let manifest = Manifest::default();
        let state = ModuleState {
            latest_mtime: 1000,
            file_count: 5,
            files_hash: 12345,
        };
        assert!(manifest.needs_regeneration("new-module", &state));
    }

    #[test]
    fn test_needs_regeneration_unchanged() {
        let mut manifest = Manifest::default();
        let state = ModuleState {
            latest_mtime: 1000,
            file_count: 5,
            files_hash: 12345,
        };
        manifest.update_module("module".to_string(), state.clone());
        assert!(!manifest.needs_regeneration("module", &state));
    }

    #[test]
    fn test_needs_regeneration_mtime_changed() {
        let mut manifest = Manifest::default();
        let old_state = ModuleState {
            latest_mtime: 1000,
            file_count: 5,
            files_hash: 12345,
        };
        manifest.update_module("module".to_string(), old_state);

        let new_state = ModuleState {
            latest_mtime: 2000,
            file_count: 5,
            files_hash: 12345,
        };
        assert!(manifest.needs_regeneration("module", &new_state));
    }

    #[test]
    fn test_needs_regeneration_file_count_changed() {
        let mut manifest = Manifest::default();
        let old_state = ModuleState {
            latest_mtime: 1000,
            file_count: 5,
            files_hash: 12345,
        };
        manifest.update_module("module".to_string(), old_state);

        let new_state = ModuleState {
            latest_mtime: 1000,
            file_count: 6,
            files_hash: 12345,
        };
        assert!(manifest.needs_regeneration("module", &new_state));
    }

    #[test]
    fn test_prune_modules() {
        let mut manifest = Manifest::default();
        manifest.update_module(
            "a".to_string(),
            ModuleState {
                latest_mtime: 0,
                file_count: 0,
                files_hash: 0,
            },
        );
        manifest.update_module(
            "b".to_string(),
            ModuleState {
                latest_mtime: 0,
                file_count: 0,
                files_hash: 0,
            },
        );
        manifest.update_module(
            "c".to_string(),
            ModuleState {
                latest_mtime: 0,
                file_count: 0,
                files_hash: 0,
            },
        );

        manifest.prune_modules(&["a".to_string(), "c".to_string()]);

        assert!(manifest.modules.contains_key("a"));
        assert!(!manifest.modules.contains_key("b"));
        assert!(manifest.modules.contains_key("c"));
    }
}
