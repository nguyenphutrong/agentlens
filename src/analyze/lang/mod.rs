mod c;
mod cpp;
mod csharp;
mod dart;
mod go;
mod java;
mod javascript;
mod php;
mod python;
mod ruby;
mod rust;

pub use c::CParser;
pub use cpp::CppParser;
pub use csharp::CSharpParser;
pub use dart::DartParser;
pub use go::GoParser;
pub use java::JavaParser;
pub use javascript::JavaScriptParser;
pub use php::PhpParser;
pub use python::PythonParser;
pub use ruby::RubyParser;
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
        Language::C => Some(Box::new(CParser)),
        Language::Cpp => Some(Box::new(CppParser)),
        Language::Ruby => Some(Box::new(RubyParser)),
        Language::Dart => Some(Box::new(DartParser)),
        Language::Unknown => None,
    }
}
