use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Update agentmap to the latest version
    Update,
    /// Watch for file changes and regenerate docs automatically
    Watch {
        /// Debounce delay in milliseconds
        #[arg(long, default_value = "300")]
        debounce: u64,
    },
    /// Manage git hooks for automatic regeneration
    Hooks {
        #[command(subcommand)]
        action: HooksAction,
    },
    /// Initialize agentmap configuration file
    Init {
        /// Create agentmap.toml with default settings
        #[arg(long)]
        config: bool,
        /// Install git hooks
        #[arg(long)]
        hooks: bool,
        /// Generate AI tool templates (cursor, claude, opencode, or all)
        #[arg(long, value_name = "TOOLS")]
        templates: Option<Option<String>>,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum HooksAction {
    /// Install git hooks (pre-commit, post-checkout, post-merge)
    Install,
    /// Remove git hooks
    Remove,
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

    /// Path to config file
    #[arg(long, value_name = "FILE")]
    pub config: Option<PathBuf>,

    /// Check if docs are stale (exit 1 if regeneration needed)
    #[arg(long, default_value = "false")]
    pub check: bool,
}

impl Args {
    pub fn with_config(mut self) -> Self {
        use crate::config::Config;

        let config_path = self.config.clone();
        let project_path = &self.path;

        let config = if let Some(path) = config_path {
            Config::load_from_path(&path)
        } else {
            Config::load(project_path)
        };

        if let Some(cfg) = config {
            if let Some(output) = cfg.output {
                if self.output.as_os_str() == ".agentmap" {
                    self.output = PathBuf::from(output);
                }
            }
            if let Some(threshold) = cfg.threshold {
                if self.threshold == 500 {
                    self.threshold = threshold;
                }
            }
            if let Some(complex) = cfg.complex_threshold {
                if self.complex_threshold == 1000 {
                    self.complex_threshold = complex;
                }
            }
            if let Some(module_depth) = cfg.module_depth {
                if self.module_depth == 3 {
                    self.module_depth = module_depth;
                }
            }
            if let Some(depth) = cfg.depth {
                if self.depth == 0 {
                    self.depth = depth;
                }
            }
            if !cfg.ignore.is_empty() && self.ignore.is_empty() {
                self.ignore = cfg.ignore;
            }
            if !cfg.lang.is_empty() && self.lang.is_empty() {
                self.lang = cfg.lang;
            }
            if let Some(no_gitignore) = cfg.no_gitignore {
                if !self.no_gitignore {
                    self.no_gitignore = no_gitignore;
                }
            }
        }

        self
    }

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
