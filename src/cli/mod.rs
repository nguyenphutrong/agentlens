mod args;
pub mod check;
mod hooks;
mod serve;
mod templates;
mod update;
mod watch;

pub use args::{Args, Command, HooksAction};
pub use check::run_check;
pub use hooks::{install_hooks, install_hooks_with_manager, remove_hooks};
pub use serve::{run_mcp_http_server, run_mcp_server};
pub use templates::run_templates;
pub use update::run_update;
pub use watch::run_watch;
