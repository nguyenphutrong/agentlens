mod args;
mod check;
mod hooks;
mod update;
mod watch;

pub use args::{Args, Command, HooksAction};
pub use check::run_check;
pub use hooks::{install_hooks, remove_hooks};
pub use update::run_update;
pub use watch::run_watch;
