use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Rust,
    Python,
    JavaScript,
    TypeScript,
    Go,
    Php,
    Java,
    CSharp,
    Unknown,
}

impl Language {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "rs" => Language::Rust,
            "py" => Language::Python,
            "js" | "jsx" | "mjs" | "cjs" => Language::JavaScript,
            "ts" | "tsx" | "mts" | "cts" => Language::TypeScript,
            "go" => Language::Go,
            "php" | "phtml" => Language::Php,
            "java" => Language::Java,
            "cs" => Language::CSharp,
            _ => Language::Unknown,
        }
    }

    pub fn from_shebang(first_line: &str) -> Option<Self> {
        if !first_line.starts_with("#!") {
            return None;
        }

        if first_line.contains("python") {
            Some(Language::Python)
        } else if first_line.contains("node")
            || first_line.contains("deno")
            || first_line.contains("bun")
        {
            Some(Language::JavaScript)
        } else if first_line.contains("php") {
            Some(Language::Php)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct FileEntry {
    pub path: PathBuf,
    pub relative_path: String,
    pub extension: Option<String>,
    pub language: Language,
    pub size_bytes: u64,
    pub line_count: usize,
    pub is_large: bool,
}

impl FileEntry {
    pub fn new(
        path: PathBuf,
        relative_path: String,
        size_bytes: u64,
        line_count: usize,
        threshold: usize,
    ) -> Self {
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .map(|s| s.to_string());

        let language = extension
            .as_ref()
            .map(|e| Language::from_extension(e))
            .unwrap_or(Language::Unknown);

        Self {
            path,
            relative_path,
            extension,
            language,
            size_bytes,
            line_count,
            is_large: line_count > threshold,
        }
    }
}
