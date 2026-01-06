mod agent;
mod file_doc;
mod imports;
mod index;
mod memory;
mod module_content;
mod outline;

pub use agent::{generate_agent_md, AgentConfig, ProjectSize};
pub use file_doc::{
    file_path_to_slug, generate_file_doc, is_complex_file, DEFAULT_COMPLEX_LINES_THRESHOLD,
    DEFAULT_COMPLEX_SYMBOLS_THRESHOLD,
};
pub use imports::generate_imports;
pub use index::{detect_entry_points, generate_index_md, IndexConfig};
pub use memory::{generate_memory, get_critical_files};
pub use module_content::generate_module_content;
pub use outline::generate_outline;
