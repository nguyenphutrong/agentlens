# agentlens

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

**Prepare codebases for AI agents** by generating structured documentation that helps AI assistants understand and navigate your code effectively.

[🇻🇳 Tiếng Việt](README.vi.md)

## What It Does

agentlens scans your codebase and generates a **hierarchical documentation structure** organized by modules:

```
.agentlens/
├── INDEX.md              # L0: Global routing table
├── modules/
│   └── {module-slug}/
│       ├── MODULE.md     # L1: Module summary
│       ├── outline.md    # L1: Symbol maps for this module
│       ├── memory.md     # L1: Warnings/TODOs for this module
│       └── imports.md    # L1: Dependencies for this module
└── files/
    └── {file-slug}.md    # L2: Deep docs for complex files (optional)
```

### Content Hierarchy

| Level | File | Purpose | Size |
|-------|------|---------|------|
| L0 | `INDEX.md` | Global routing table with module overview | O(modules) |
| L1 | `MODULE.md` | Module summary, file list, entry points | O(files in module) |
| L1 | `outline.md` | Symbol maps for large files in module | O(large files) |
| L1 | `memory.md` | Warnings and TODOs scoped to module | O(markers) |
| L1 | `imports.md` | Intra/inter-module dependencies | O(imports) |
| L2 | `files/*.md` | Deep documentation for complex files | O(symbols) |

## Why?

AI coding assistants struggle with large codebases because they can't see the full picture. agentlens provides:

- **Hierarchical navigation** so AI loads only what it needs (not entire codebase docs)
- **Module detection** that groups files into semantic units automatically
- **Symbol maps** so AI knows what's in large files without reading them entirely
- **Scoped context** so each module's docs contain only relevant information

## Installation

### For Humans

#### Homebrew (macOS) — Recommended

```bash
brew install nguyenphutrong/tap/agentlens
```

#### npm / pnpm / yarn / bun

```bash
npx @agentlens/cli            # Run without install
npm install -g @agentlens/cli
pnpm add -g @agentlens/cli
yarn global add @agentlens/cli
bun add -g @agentlens/cli
```

#### Quick Install Script

```bash
curl -fsSL https://raw.githubusercontent.com/nguyenphutrong/agentlens/main/scripts/install.sh | sh
```

#### From crates.io

```bash
cargo install agentlens
```

#### From Source

```bash
git clone https://github.com/nguyenphutrong/agentlens
cd agentlens
cargo install --path .
```

#### Manual Download

Download prebuilt binaries from [GitHub Releases](https://github.com/nguyenphutrong/agentlens/releases).

### Alternative: Let an LLM Agent Do It

Copy this prompt to your AI coding assistant:

```
Install and configure agentlens by following the instructions at:
https://github.com/nguyenphutrong/agentlens/blob/main/docs/ai-agent-setup.md
```

## Usage

### Basic

```bash
# Generate docs for current directory (hierarchical mode - default)
agentlens

# Output to custom directory
agentlens -o docs/ai

# Preview without writing files
agentlens --dry-run

# Verbose output
agentlens -v
```

### Remote Repositories

```bash
# Analyze a GitHub repository directly
agentlens github.com/user/repo
agentlens https://github.com/vercel/next.js

# With depth limiting for large repos
agentlens --depth 3 github.com/facebook/react
```

### Git Diff Mode

```bash
# Show only files changed since a branch
agentlens --diff main

# Compare against a specific commit
agentlens --diff HEAD~5
```

### JSON Output

```bash
# Output analysis as JSON (for tooling integration)
agentlens --json > analysis.json

# Combine with other flags
agentlens --json --depth 2 github.com/user/repo
```

### Options

```
Usage: agentlens [OPTIONS] [PATH]

Arguments:
  [PATH]  Target directory or GitHub URL [default: .]

Options:
  -o, --output <OUTPUT>              Output directory [default: .agentlens]
  -t, --threshold <THRESHOLD>        Line threshold for "large" files [default: 500]
  -c, --complex-threshold <COMPLEX>  Symbol threshold for L2 file docs [default: 30]
  -d, --depth <DEPTH>                Max directory depth (0 = unlimited) [default: 0]
      --diff <REF>                   Compare against git branch/commit
      --json                         Output JSON to stdout instead of markdown files
      --check                        Check if docs are stale (exit 1 if regeneration needed)
      --config <FILE>                Path to config file
      --force                        Force regenerate all modules (ignore cache)
  -i, --ignore <IGNORE>              Additional patterns to ignore
  -l, --lang <LANG>                  Filter by language
      --no-gitignore                 Don't respect .gitignore
      --dry-run                      Preview output without writing
  -v, --verbose...                   Increase verbosity (-v, -vv, -vvv)
  -q, --quiet                        Suppress all output
  -h, --help                         Print help
  -V, --version                      Print version

Commands:
  watch   Watch for file changes and regenerate docs automatically
  hooks   Manage git hooks for automatic regeneration
  init    Initialize agentlens configuration
  update  Update agentlens to the latest version
```

## Watch Mode

Keep documentation in sync automatically during development:

```bash
# Start watching for changes (regenerates on file save)
agentlens watch

# Custom debounce delay (default: 300ms)
agentlens watch --debounce 500
```

Watch mode leverages the incremental manifest system, so only changed modules are regenerated.

## Git Hooks

Automate documentation regeneration at key git events:

```bash
# Install hooks (auto-detects Husky, Lefthook, pre-commit, or native git)
agentlens hooks install

# Force specific hook manager
agentlens hooks install --native      # Native git hooks
agentlens hooks install --husky       # Husky (.husky/)
agentlens hooks install --lefthook    # Lefthook (lefthook.yml)
agentlens hooks install --pre-commit  # pre-commit (.pre-commit-config.yaml)

# Remove hooks
agentlens hooks remove

# Skip hooks temporarily
AGENTLENS_SKIP=1 git commit -m "quick fix"
```

### Supported Hook Managers

| Manager | Detection | Hooks Supported |
|---------|-----------|-----------------|
| **Native git** | Fallback | pre-commit, post-checkout, post-merge |
| **Husky** | `.husky/` directory | pre-commit, post-checkout, post-merge |
| **Lefthook** | `lefthook.yml` | pre-commit, post-checkout, post-merge |
| **pre-commit** | `.pre-commit-config.yaml` | pre-commit only |

Installed hooks:
- **pre-commit**: Regenerates docs and stages `.agentlens/`
- **post-checkout**: Regenerates after branch switch (background)
- **post-merge**: Regenerates after pull/merge (background)

## Best Practices

### Should I commit `.agentlens/`?

| Team Size | Branches | Recommendation |
|-----------|----------|----------------|
| Solo / Small (1-5) | Few | **Commit** - docs available immediately on clone |
| Medium (5-15) | Many | **Ignore** - avoid merge conflicts |
| Large (15+) | Many | **Ignore** - use CI to validate freshness |
| Open Source Library | Any | **Commit** - showcase output for contributors |

### Approach A: Commit `.agentlens/` (Small teams / OSS)

Best when you want docs available immediately after `git clone`.

```bash
# Install hooks to keep docs synced across branches
agentlens hooks install
```

Add to `.gitattributes` to reduce merge conflicts:

```gitattributes
.agentlens/** merge=ours -diff
```

### Approach B: Ignore `.agentlens/` (Larger teams)

Best when multiple developers work on many branches.

```gitignore
# .gitignore
.agentlens/
```

Each developer runs once after cloning:

```bash
agentlens hooks install
```

Docs auto-regenerate on `git checkout`, `git pull`, and `git commit`.

### Approach C: CI-only (Strict freshness)

Generate docs in CI, never commit locally:

```gitignore
# .gitignore
.agentlens/
```

```yaml
# .github/workflows/docs.yml
- name: Generate agentlens docs
  run: agentlens
- name: Upload as artifact
  uses: actions/upload-artifact@v4
  with:
    name: agentlens-docs
    path: .agentlens/
```

## Configuration File

Create `agentlens.toml` for project-specific settings:

```bash
# Generate default config file
agentlens init --config

# Use custom config location
agentlens --config custom.toml
```

Example `agentlens.toml`:

```toml
output = ".agentlens"
threshold = 500
complex_threshold = 1000
ignore = ["*.test.ts", "fixtures/", "__mocks__/"]

[watch]
debounce_ms = 300
```

CLI flags override config file values.

## AI Tool Templates

Generate ready-to-use configuration templates for popular AI coding tools:

```bash
# Generate all templates (.cursorrules, CLAUDE.md, AGENTS.md)
agentlens init --templates

# Generate specific templates only
agentlens init --templates=cursor
agentlens init --templates=claude,opencode
```

Supported tools:
- **Cursor** (`.cursorrules`) - Instructions for Cursor IDE
- **Claude Code** (`CLAUDE.md`) - Instructions for Claude Code
- **OpenCode** (`AGENTS.md`) - Instructions for OpenCode

Templates are **non-destructive**: they append to existing files and skip if agentlens section already exists.

## MCP Server

agentlens can run as an MCP server for AI tools like Claude Desktop and Cursor:

```bash
# Using npx (no install required)
npx @agentlens/cli serve --mcp

# Or if installed globally
agentlens serve --mcp
```

**Available tools:**

| Tool | Description |
|------|-------------|
| `regenerate` | Regenerate documentation |
| `get_module` | Get module docs by slug |
| `check_stale` | Check if docs need update |
| `get_outline` | Get symbol outline for a file |

**Example MCP config (Claude Desktop, Cursor, etc.):**

```json
{
  "mcpServers": {
    "agentlens": {
      "command": "npx",
      "args": ["@agentlens/cli", "serve", "--mcp"]
    }
  }
}
```

See [MCP Server Documentation](docs/mcp-server.md) for setup instructions and integration guides.

## CI Integration

Validate documentation freshness in CI pipelines:

```bash
# Check if docs are stale (exit 0 = fresh, exit 1 = stale)
agentlens --check

# Combine with diff mode
agentlens --check --diff main
```

Example GitHub Actions workflow:

```yaml
name: Check Agentmap
on: [pull_request]

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install agentlens
        run: cargo install agentlens
      - name: Check docs freshness
        run: agentlens --check
```

## Module Detection

agentlens automatically detects module boundaries using language-specific conventions:

| Language | Explicit Boundary | Example |
|----------|-------------------|---------|
| Rust | `mod.rs`, `lib.rs` | `src/analyze/mod.rs` → module `src-analyze` |
| Python | `__init__.py` | `src/utils/__init__.py` → module `src-utils` |
| JavaScript/TypeScript | `index.{js,ts,tsx}` | `src/components/index.ts` → module `src-components` |
| Any | 5+ source files in directory | `src/helpers/` with 5+ files → implicit module |

### Module Slug Naming

Directory paths are converted to slugs using hyphens:
- `src/analyze/lang` → `src-analyze-lang`
- `lib/utils` → `lib-utils`

## Example Output

### INDEX.md (L0 Global)

```markdown
# Project

## Reading Protocol

**Start here**, then navigate to specific modules.

1. Read this INDEX for overview
2. Go to relevant `modules/{name}/MODULE.md`
3. Check module's `outline.md` for large files
4. Check module's `memory.md` for warnings

## Entry Points

- `src/main.rs`
- `src/lib.rs`

## Modules

| Module | Type | Files | Warnings | Hub |
| ------ | ---- | ----- | -------- | --- |
| [src](modules/src/MODULE.md) | rust | 2 | - |  |
| [src/analyze](modules/src-analyze/MODULE.md) | rust | 5 | ⚠️ 2 |  |
| [src/generate](modules/src-generate/MODULE.md) | rust | 8 | - | 🔗 |
```

### MODULE.md (L1 Module)

```markdown
# Module: src/analyze

[← Back to INDEX](../../INDEX.md)

**Type:** rust | **Files:** 5

**Entry point:** `src/analyze/mod.rs`

## Files

| File | Lines | Large |
| ---- | ----- | ----- |
| `src/analyze/graph.rs` | 98 |  |
| `src/analyze/parser.rs` | 650 | 📄 |
| `src/analyze/mod.rs` | 10 |  |

## Child Modules

- [src-analyze-lang](../src-analyze-lang/MODULE.md)

## Documentation

- [outline.md](outline.md) - Symbol maps for large files
- [memory.md](memory.md) - Warnings and TODOs
- [imports.md](imports.md) - Dependencies
```

### outline.md (L1 Module-Scoped)

```markdown
# Outline: src/analyze

[← MODULE.md](MODULE.md) | [← INDEX.md](../../INDEX.md)

## src/analyze/parser.rs (650 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 15 | fn | parse_symbols | pub |
| 89 | fn | extract_functions | (private) |
| 156 | struct | ParseResult | pub |
```

### memory.md (L1 Module-Scoped)

```markdown
# Memory: src/analyze

[← MODULE.md](MODULE.md) | [← INDEX.md](../../INDEX.md)

## ⚠️ Warnings

### 🔴 `WARNING` (src/analyze/parser.rs:42)
> Edge case not handled for nested generics

## 🔧 Technical Debt

### 🟡 `TODO` (src/analyze/graph.rs:128)
> Optimize cycle detection algorithm
```

## Supported Languages

| Language | Symbol Extraction | Import Graph | Memory Markers | Module Detection |
|----------|-------------------|--------------|----------------|------------------|
| Rust | ✅ Functions, structs, enums, traits, impls | ✅ | ✅ | ✅ `mod.rs` |
| Python | ✅ Functions, classes, methods | ✅ | ✅ | ✅ `__init__.py` |
| JavaScript/TypeScript | ✅ Functions, classes, arrow functions | ✅ | ✅ | ✅ `index.{js,ts}` |
| Go | ✅ Functions, structs, interfaces, methods | ✅ | ✅ | ✅ implicit |
| Swift | ✅ Functions, classes, structs, enums, protocols | ✅ | ✅ | ✅ implicit |
| Dart | ✅ Functions, classes, mixins, extensions | ✅ | ✅ | ✅ implicit |
| Ruby | ✅ Methods, classes, modules | ✅ | ✅ | ✅ implicit |
| C# | ✅ Methods, classes, structs, interfaces | ✅ | ✅ | ✅ implicit |
| Java | ✅ Methods, classes, interfaces, enums | ✅ | ✅ | ✅ implicit |

## Memory Markers

agentlens extracts these comment patterns:

| Pattern | Category | Priority |
|---------|----------|----------|
| `TODO`, `FIXME`, `XXX`, `BUG`, `HACK` | Technical Debt | Medium |
| `WARNING`, `WARN` | Warnings | High |
| `SAFETY`, `INVARIANT` | Safety | High |
| `RULE`, `POLICY` | Business Rules | High |
| `DEPRECATED` | Technical Debt | High |
| `NOTE` | Notes | Low |

## Integration with AI Tools

### Claude Code / Cursor

Add to your project's AI instructions:

```
Before working on this codebase, read:
1. .agentlens/INDEX.md - for project overview and module routing
2. Navigate to relevant module's MODULE.md for details
3. Check module's memory.md for warnings before editing
4. Consult module's outline.md for large file navigation
```

### GitHub Copilot

Include `.agentlens/` in your workspace context.

### JSON Integration

For programmatic access:

```bash
agentlens --json | jq '.modules[] | {slug, file_count, warning_count}'
```

JSON output includes:
- `modules[]` - Array of module metadata (slug, path, file_count, warning_count, symbol_count, is_hub)
- `files[]` - All scanned files with metadata
- `memory[]` - All memory markers with locations
- `entry_points[]` - Detected entry points
- `hub_files[]` - Files imported by 3+ others

## Development

```bash
# Run tests
cargo test

# Run with verbose output
cargo run -- -vv .

# Check for issues
cargo clippy
```

## License

MIT License - see [LICENSE](LICENSE) for details.
