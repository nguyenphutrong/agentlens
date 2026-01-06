use crate::types::Language;
use std::path::Path;

pub fn should_include_file(path: &Path, allowed_languages: &[String]) -> bool {
    let extension = match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => ext,
        None => return false,
    };

    let language = Language::from_extension(extension);

    if matches!(language, Language::Unknown) {
        return false;
    }

    if allowed_languages.is_empty() {
        return true;
    }

    let lang_name = match language {
        Language::Rust => "rust",
        Language::Python => "python",
        Language::JavaScript => "javascript",
        Language::TypeScript => "typescript",
        Language::Go => "go",
        Language::Php => "php",
        Language::Java => "java",
        Language::CSharp => "csharp",
        Language::Unknown => return false,
    };

    allowed_languages
        .iter()
        .any(|l| l.to_lowercase() == lang_name)
}
