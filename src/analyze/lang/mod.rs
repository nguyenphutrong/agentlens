mod csharp;
mod go;
mod java;
mod javascript;
mod php;
mod python;
mod rust;

pub use csharp::CSharpParser;
pub use go::GoParser;
pub use java::JavaParser;
pub use javascript::JavaScriptParser;
pub use php::PhpParser;
pub use python::PythonParser;
pub use rust::RustParser;

use crate::types::{Language, Symbol};

pub trait LanguageParser {
    fn parse_symbols(&self, content: &str) -> Vec<Symbol>;
}

pub fn get_parser(language: Language) -> Option<Box<dyn LanguageParser>> {
    match language {
        Language::Rust => Some(Box::new(RustParser)),
        Language::Python => Some(Box::new(PythonParser)),
        Language::JavaScript | Language::TypeScript => Some(Box::new(JavaScriptParser)),
        Language::Go => Some(Box::new(GoParser)),
        Language::Php => Some(Box::new(PhpParser)),
        Language::Java => Some(Box::new(JavaParser)),
        Language::CSharp => Some(Box::new(CSharpParser)),
        Language::Unknown => None,
    }
}
