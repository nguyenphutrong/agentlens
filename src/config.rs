use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

const CONFIG_FILE_NAME: &str = "agentmap.toml";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    pub output: Option<String>,
    pub threshold: Option<usize>,
    pub complex_threshold: Option<usize>,
    pub module_depth: Option<usize>,
    pub depth: Option<usize>,
    #[serde(default)]
    pub ignore: Vec<String>,
    #[serde(default)]
    pub lang: Vec<String>,
    pub no_gitignore: Option<bool>,
    pub watch: Option<WatchConfig>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct WatchConfig {
    pub debounce_ms: Option<u64>,
}

impl Config {
    pub fn load(project_path: &Path) -> Option<Self> {
        let config_path = find_config_file(project_path)?;
        let content = fs::read_to_string(&config_path).ok()?;
        toml::from_str(&content).ok()
    }

    pub fn load_from_path(config_path: &Path) -> Option<Self> {
        let content = fs::read_to_string(config_path).ok()?;
        toml::from_str(&content).ok()
    }

    pub fn generate_default() -> String {
        r#"# agentmap configuration
# See https://github.com/nguyenphutrong/agentmap for documentation

# Output directory for generated documentation
# output = ".agentmap"

# Line threshold for "large" files (generates outline)
# threshold = 500

# Line threshold for L2 file-level docs (very complex files)
# complex_threshold = 1000

# Maximum module nesting depth (0 = unlimited)
# module_depth = 3

# Maximum directory depth (0 = unlimited)  
# depth = 0

# Additional patterns to ignore (in addition to .gitignore)
# ignore = ["*.test.ts", "fixtures/", "__mocks__/"]

# Filter by language (empty = all languages)
# lang = ["rust", "typescript"]

# Don't respect .gitignore
# no_gitignore = false

# Watch mode configuration
# [watch]
# debounce_ms = 300
"#
        .to_string()
    }

    pub fn create_default_file(project_path: &Path) -> std::io::Result<PathBuf> {
        let config_path = project_path.join(CONFIG_FILE_NAME);
        fs::write(&config_path, Self::generate_default())?;
        Ok(config_path)
    }
}

fn find_config_file(start_path: &Path) -> Option<PathBuf> {
    let mut current = start_path.to_path_buf();
    loop {
        let config_path = current.join(CONFIG_FILE_NAME);
        if config_path.exists() {
            return Some(config_path);
        }
        if !current.pop() {
            break;
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_parse_minimal_config() {
        let config: Config = toml::from_str("").unwrap();
        assert!(config.output.is_none());
        assert!(config.threshold.is_none());
    }

    #[test]
    fn test_parse_full_config() {
        let content = r#"
output = ".docs"
threshold = 300
complex_threshold = 500
ignore = ["test/", "*.spec.ts"]

[watch]
debounce_ms = 500
"#;
        let config: Config = toml::from_str(content).unwrap();
        assert_eq!(config.output, Some(".docs".to_string()));
        assert_eq!(config.threshold, Some(300));
        assert_eq!(config.complex_threshold, Some(500));
        assert_eq!(config.ignore.len(), 2);
        assert_eq!(config.watch.unwrap().debounce_ms, Some(500));
    }

    #[test]
    fn test_find_config_file() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        fs::write(&config_path, "output = \".test\"").unwrap();

        let found = find_config_file(temp.path());
        assert!(found.is_some());
        assert_eq!(found.unwrap(), config_path);
    }

    #[test]
    fn test_find_config_file_in_parent() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join(CONFIG_FILE_NAME);
        fs::write(&config_path, "output = \".test\"").unwrap();

        let subdir = temp.path().join("src").join("nested");
        fs::create_dir_all(&subdir).unwrap();

        let found = find_config_file(&subdir);
        assert!(found.is_some());
        assert_eq!(found.unwrap(), config_path);
    }

    #[test]
    fn test_create_default_file() {
        let temp = TempDir::new().unwrap();
        let result = Config::create_default_file(temp.path());
        assert!(result.is_ok());

        let config_path = result.unwrap();
        assert!(config_path.exists());

        let content = fs::read_to_string(&config_path).unwrap();
        assert!(content.contains("agentmap configuration"));
    }
}
