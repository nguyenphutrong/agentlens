# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.7.0] - 2026-01-22

### Added

- **Semantic search** - AI-powered code search with natural language queries:
  - `agentlens index` - Build vector index from codebase
  - `agentlens search "query"` - Search with natural language
  - Symbol-based chunking with sliding window fallback
  - Hybrid search mode (vector + text with RRF fusion)
- **Ollama embedder** - Local embedding generation via Ollama API (`nomic-embed-text` model, 768 dimensions)
- **MCP semantic_search tool** - Expose search capabilities to AI tools via Model Context Protocol
- **Mermaid diagrams** - Visualize module dependencies with `graph TD` diagrams in `INDEX.md`
- **Interactive install TUI** - Guided setup wizard for first-time users
- **Skills subcommand** (`agentlens skills`) - Manage AI tool skill configurations
- **Token counting** - Usage analytics and telemetry module for tracking token consumption
- **Code of Conduct** - Contributor Covenant added

### Changed

- Renamed `AgentmapServer` to `AgentlensServer` for consistency
- Updated template markers to use `agentlens` naming

### Fixed

- Clippy warnings across codebase
- Code formatting with `cargo fmt`

### Configuration

New `[search]` section in `agentlens.toml`:

```toml
[search]
[search.embedder]
provider = "ollama"
model = "nomic-embed-text"
dimensions = 768

[search.chunking]
max_tokens = 512
strategy = "symbol"

[search.search]
hybrid_enabled = true
hybrid_k = 60.0
default_limit = 10
```

## [0.4.2] - 2026-01-07

### Added

- **Multiple git hook managers** - Auto-detect and integrate with:
  - Husky (`.husky/` directory)
  - Lefthook (`lefthook.yml`)
  - pre-commit (`.pre-commit-config.yaml`)
  - Native git hooks (`.git/hooks/`)
- **CLI flags for hook managers** - `--native`, `--husky`, `--lefthook`, `--pre-commit`
- **npm package** (`@agentlens/cli`) - Install via `npm install -g @agentlens/cli` or use `npx @agentlens/cli`
- **MCP server documentation** - Comprehensive docs (EN/VI) for Model Context Protocol integration

### Changed

- npm package now has dedicated README with usage examples

## [0.4.0] - 2026-01-07

### Added

- **MCP server** - Model Context Protocol support with stdio transport for AI tool integration
- **AI tool templates** - Pre-built configurations for Cursor, Claude Code, OpenCode
- **AGENT.md generation** - Auto-generated AI instruction file at project root
- **Watch mode** (`--watch`) - Real-time file watching with automatic regeneration
- **Git hooks** (`--hooks`) - Automatic regeneration on git commit
- **Config file** (`agentlens.toml`) - Project-level configuration support
- **CI validation** (`--check`) - Exit non-zero if `.agentlens/` is outdated
- **Self-update** (`--update`) - Update agentlens binary from command line
- **Homebrew tap** - Install via `brew install trongnguyen24/tap/agentlens`

### Fixed

- Clippy warnings in args and runner modules

## [0.3.0] - 2026-01-06

### Added

- **Smart file generation** - Reduce file clutter for small/medium projects:
  - Skip empty `outline.md` when no large files exist
  - Skip empty `memory.md` when no TODO/WARNING markers exist
  - Auto-merge small content (<500 bytes) inline into `MODULE.md`
- **CI/CD** - Automatic publish to crates.io on release

### Changed

- `MODULE.md` now conditionally shows Documentation section (only if separate files exist)
- Small imports/outline/memory content inlined with `---` separator

### Performance

- Up to **70% reduction** in generated files for typical projects (33 → 10 files)

## [0.2.0] - 2026-01-06

Initial release.

### Added

- **Hierarchical content architecture** - Three-level documentation structure (L0/L1/L2) for scalability
- **Module detection** - Automatic boundary detection via `mod.rs`, `__init__.py`, `index.{js,ts}`, or 5+ files
- **9 language support** - Rust, Python, JavaScript/TypeScript, Go, Swift, Dart, Ruby, C#, Java
- **Symbol extraction** - Functions, classes, structs, traits, enums, interfaces
- **Memory marker detection** - TODO, FIXME, WARNING, SAFETY, RULE, DEPRECATED, etc.
- **Hub file detection** - Identify high-impact files imported by 3+ others
- **Import graph visualization** - Show intra/inter-module dependencies
- **Git diff mode** (`--diff`) - Filter output to changed files only
- **JSON output** (`--json`) - Machine-readable output for tooling integration
- **Remote repository support** - Analyze GitHub repositories directly via URL
- **Incremental regeneration** - Only update modules that changed
- **CLI options**: `-o`, `-t`, `-c`, `-d`, `-i`, `-l`, `-v`, `--dry-run`, `--no-gitignore`

### Output Structure

```
.agentlens/
├── INDEX.md              # L0: Global routing table
├── modules/
│   └── {module-slug}/
│       ├── MODULE.md     # L1: Module summary
│       ├── outline.md    # L1: Symbol maps (if large files exist)
│       ├── memory.md     # L1: Warnings/TODOs (if markers exist)
│       └── imports.md    # L1: Dependencies (inline if small)
└── files/
    └── {file-slug}.md    # L2: Deep docs for complex files
```
