pub mod graph;
pub mod lang;
mod memory;
pub mod module;
mod parser;

pub use graph::FileGraph;
pub use memory::extract_memory_markers;
pub use module::{detect_modules, path_to_slug, BoundaryType, ModuleInfo};
pub use parser::{extract_imports, extract_symbols};
