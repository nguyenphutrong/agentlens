use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Update agentmap to the latest version
    Update,
}

#[derive(Parser, Debug)]
#[command(name = "agentmap")]
#[command(version)]
#[command(about = "Prepare codebases for AI agents by generating hierarchical documentation")]
#[command(long_about = "
agentmap scans your codebase and generates a hierarchical documentation structure:

  .agentmap/
  ├── INDEX.md              # Global routing table (constant size)
  ├── modules/{module}/     # Per-module documentation
  │   ├── MODULE.md         # Module overview and file list
  │   ├── outline.md        # Symbol maps for large files
  │   ├── memory.md         # TODOs, warnings, business rules
  │   └── imports.md        # Dependencies within module
  └── files/{slug}.md       # Deep docs for complex files (L2)

This hierarchical structure scales O(1) per module, enabling AI agents to
efficiently navigate large codebases without context overflow.
")]
#[command(author = "AgentMap Contributors")]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Target directory or GitHub URL
    #[arg(default_value = ".")]
    pub path: PathBuf,

    /// Output directory for generated documentation
    #[arg(short, long, default_value = ".agentmap")]
    pub output: PathBuf,

    /// Line threshold for "large" files (generates outline)
    #[arg(short, long, default_value = "500")]
    pub threshold: usize,

    /// Line threshold for L2 file-level docs (very complex files)
    #[arg(long, default_value = "1000", value_name = "LINES")]
    pub complex_threshold: usize,

    /// Maximum module nesting depth (0 = unlimited)
    #[arg(long, default_value = "3", value_name = "DEPTH")]
    pub module_depth: usize,

    /// Additional patterns to ignore
    #[arg(short, long, action = clap::ArgAction::Append)]
    pub ignore: Vec<String>,

    /// Filter by language
    #[arg(short, long, action = clap::ArgAction::Append)]
    pub lang: Vec<String>,

    /// Don't respect .gitignore
    #[arg(long, default_value = "false")]
    pub no_gitignore: bool,

    /// Preview output without writing files
    #[arg(long, default_value = "false")]
    pub dry_run: bool,

    /// Increase verbosity (-v, -vv, -vvv)
    #[arg(short, long, action = clap::ArgAction::Count)]
    pub verbose: u8,

    /// Suppress all output
    #[arg(short, long, default_value = "false")]
    pub quiet: bool,

    /// Compare against git branch/commit
    #[arg(long, value_name = "REF")]
    pub diff: Option<String>,

    /// Output JSON to stdout instead of markdown files
    #[arg(long, default_value = "false")]
    pub json: bool,

    /// Max directory depth (0 = unlimited)
    #[arg(short = 'd', long, default_value = "0")]
    pub depth: usize,

    /// Force regenerate all modules (ignore cache)
    #[arg(long, default_value = "false")]
    pub force: bool,
}

impl Args {
    pub fn verbosity(&self) -> u8 {
        if self.quiet {
            0
        } else {
            self.verbose + 1
        }
    }

    pub fn validate(&self) -> Result<(), String> {
        let path_str = self.path.to_string_lossy();
        if path_str.starts_with("https://")
            || path_str.starts_with("github.com")
            || path_str.starts_with("gitlab.com")
            || path_str.starts_with("git@")
        {
            if self.threshold == 0 {
                return Err("Threshold must be greater than 0".to_string());
            }
            return Ok(());
        }

        if !self.path.exists() {
            return Err(format!("Path does not exist: {}", self.path.display()));
        }

        if !self.path.is_dir() {
            return Err(format!("Path is not a directory: {}", self.path.display()));
        }

        if self.threshold == 0 {
            return Err("Threshold must be greater than 0".to_string());
        }

        Ok(())
    }

    pub fn is_remote(&self) -> bool {
        let path_str = self.path.to_string_lossy();
        path_str.starts_with("https://")
            || path_str.starts_with("github.com")
            || path_str.starts_with("gitlab.com")
            || path_str.starts_with("git@")
    }
}
