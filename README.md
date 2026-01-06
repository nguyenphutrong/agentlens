# agentmap

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

**Prepare codebases for AI agents** by generating structured documentation that helps AI assistants understand and navigate your code effectively.

[🇻🇳 Tiếng Việt](README.vi.md)

## What It Does

agentmap scans your codebase and generates a **hierarchical documentation structure** organized by modules:

```
.agentmap/
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

AI coding assistants struggle with large codebases because they can't see the full picture. agentmap provides:

- **Hierarchical navigation** so AI loads only what it needs (not entire codebase docs)
- **Module detection** that groups files into semantic units automatically
- **Symbol maps** so AI knows what's in large files without reading them entirely
- **Scoped context** so each module's docs contain only relevant information

## Installation

### Quick Install (Recommended)

```bash
curl -fsSL https://raw.githubusercontent.com/nguyenphutrong/agentmap/main/scripts/install.sh | sh
```

### From crates.io

```bash
cargo install agentmap
```

### From Source

```bash
git clone https://github.com/nguyenphutrong/agentmap
cd agentmap
cargo install --path .
```

### Manual Download

Download prebuilt binaries from [GitHub Releases](https://github.com/nguyenphutrong/agentmap/releases).

## Usage

### Basic

```bash
# Generate docs for current directory (hierarchical mode - default)
agentmap

# Output to custom directory
agentmap -o docs/ai

# Preview without writing files
agentmap --dry-run

# Verbose output
agentmap -v
```

### Remote Repositories

```bash
# Analyze a GitHub repository directly
agentmap github.com/user/repo
agentmap https://github.com/vercel/next.js

# With depth limiting for large repos
agentmap --depth 3 github.com/facebook/react
```

### Git Diff Mode

```bash
# Show only files changed since a branch
agentmap --diff main

# Compare against a specific commit
agentmap --diff HEAD~5
```

### JSON Output

```bash
# Output analysis as JSON (for tooling integration)
agentmap --json > analysis.json

# Combine with other flags
agentmap --json --depth 2 github.com/user/repo
```

### Options

```
Usage: agentmap [OPTIONS] [PATH]

Arguments:
  [PATH]  Target directory or GitHub URL [default: .]

Options:
  -o, --output <OUTPUT>              Output directory [default: .agentmap]
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
  init    Initialize agentmap configuration
  update  Update agentmap to the latest version
```

## Watch Mode

Keep documentation in sync automatically during development:

```bash
# Start watching for changes (regenerates on file save)
agentmap watch

# Custom debounce delay (default: 300ms)
agentmap watch --debounce 500
```

Watch mode leverages the incremental manifest system, so only changed modules are regenerated.

## Git Hooks

Automate documentation regeneration at key git events:

```bash
# Install hooks (pre-commit, post-checkout, post-merge)
agentmap hooks install

# Remove hooks
agentmap hooks remove

# Skip hooks temporarily
AGENTMAP_SKIP=1 git commit -m "quick fix"
```

Installed hooks:
- **pre-commit**: Regenerates docs and stages `.agentmap/`
- **post-checkout**: Regenerates after branch switch (background)
- **post-merge**: Regenerates after pull/merge (background)

## Configuration File

Create `agentmap.toml` for project-specific settings:

```bash
# Generate default config file
agentmap init --config

# Use custom config location
agentmap --config custom.toml
```

Example `agentmap.toml`:

```toml
output = ".agentmap"
threshold = 500
complex_threshold = 1000
ignore = ["*.test.ts", "fixtures/", "__mocks__/"]

[watch]
debounce_ms = 300
```

CLI flags override config file values.

## CI Integration

Validate documentation freshness in CI pipelines:

```bash
# Check if docs are stale (exit 0 = fresh, exit 1 = stale)
agentmap --check

# Combine with diff mode
agentmap --check --diff main
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
      - name: Install agentmap
        run: cargo install agentmap
      - name: Check docs freshness
        run: agentmap --check
```

## Module Detection

agentmap automatically detects module boundaries using language-specific conventions:

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

agentmap extracts these comment patterns:

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
1. .agentmap/INDEX.md - for project overview and module routing
2. Navigate to relevant module's MODULE.md for details
3. Check module's memory.md for warnings before editing
4. Consult module's outline.md for large file navigation
```

### GitHub Copilot

Include `.agentmap/` in your workspace context.

### JSON Integration

For programmatic access:

```bash
agentmap --json | jq '.modules[] | {slug, file_count, warning_count}'
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
