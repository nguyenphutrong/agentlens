use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Update agentlens to the latest version
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
    /// Initialize agentlens configuration file
    Init {
        /// Create agentlens.toml with default settings
        #[arg(long)]
        config: bool,
        /// Install git hooks
        #[arg(long)]
        hooks: bool,
        /// Generate AI tool templates (cursor, claude, opencode, or all)
        #[arg(long, value_name = "TOOLS")]
        templates: Option<Option<String>>,
        /// Skip interactive mode, use defaults
        #[arg(long, short = 'y')]
        yes: bool,
    },
    /// Start MCP server for AI tool integration
    Serve {
        /// Run in MCP mode (stdio transport)
        #[arg(long)]
        mcp: bool,
        /// HTTP port for SSE transport (enables HTTP mode)
        #[arg(long, value_name = "PORT")]
        port: Option<u16>,
    },
    /// Analyze token usage and efficiency of generated docs
    Telemetry {
        #[command(subcommand)]
        action: TelemetryAction,
    },
    /// Manage agent skills for AI coding tools
    Skills {
        #[command(subcommand)]
        action: SkillsAction,
    },
    /// Build semantic search index for the codebase
    Index {
        #[command(subcommand)]
        action: Option<IndexAction>,
        /// Force re-index all files (ignore cache)
        #[arg(long)]
        force: bool,
        /// Prune deleted files from index
        #[arg(long)]
        prune: bool,
    },
    /// Semantic search across the codebase
    Search {
        /// Natural language query
        query: String,
        /// Number of results to return
        #[arg(short = 'n', long, default_value = "10")]
        limit: usize,
        /// Enable hybrid search (vector + text)
        #[arg(long)]
        hybrid: bool,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum HooksAction {
    /// Install git hooks (pre-commit, post-checkout, post-merge)
    Install {
        /// Force native git hooks (skip auto-detection)
        #[arg(long)]
        native: bool,
        /// Force Husky integration
        #[arg(long)]
        husky: bool,
        /// Force Lefthook integration
        #[arg(long)]
        lefthook: bool,
        /// Force pre-commit (Python) integration
        #[arg(long, name = "pre-commit")]
        pre_commit: bool,
    },
    /// Remove git hooks
    Remove,
}

#[derive(Subcommand, Debug, Clone)]
pub enum TelemetryAction {
    /// Show token usage summary for generated docs
    Summary,
    /// Analyze a specific module's token cost
    Module {
        #[arg(value_name = "SLUG")]
        slug: String,
    },
}

#[derive(Subcommand, Debug, Clone)]
pub enum SkillsAction {
    /// Install agentlens skill for AI coding tools
    Install {
        /// Install for Claude Code (~/.claude/skills/)
        #[arg(long)]
        claude: bool,
        /// Install for OpenCode (~/.config/opencode/skill/)
        #[arg(long)]
        opencode: bool,
        /// Install for Codex CLI (~/.codex/skills/)
        #[arg(long)]
        codex: bool,
        /// Install for all supported tools
        #[arg(long)]
        all: bool,
    },
    /// Remove agentlens skill from all locations
    Remove,
    /// List installed skills and their locations
    List,
}

#[derive(Subcommand, Debug, Clone)]
pub enum IndexAction {
    /// Show index status and statistics
    Status,
    /// Clear the search index
    Clear,
}

#[derive(Parser, Debug, Clone)]
#[command(name = "agentlens")]
#[command(version)]
#[command(about = "Prepare codebases for AI agents by generating hierarchical documentation")]
#[command(long_about = "
agentlens scans your codebase and generates a hierarchical documentation structure:

  .agentlens/
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
    #[arg(short, long, default_value = ".agentlens")]
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
                if self.output.as_os_str() == ".agentlens" {
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
