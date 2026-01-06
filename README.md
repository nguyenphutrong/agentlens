# agentmap

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

**Prepare codebases for AI agents** by generating structured documentation that helps AI assistants understand and navigate your code effectively.

[🇻🇳 Tiếng Việt](README.vi.md)

## What It Does

agentmap scans your codebase and generates four files:

| File | Purpose |
|------|---------
| `outline.md` | Symbol maps for large files (functions, classes, structs with line numbers) |
| `memory.md` | Extracted knowledge markers (TODO, FIXME, WARNING, SAFETY, business rules) |
| `imports.md` | File dependency graph showing imports and importers for each file |
| `AGENTS.md` | Reading instructions for AI agents (entry points, hub files, critical warnings) |

## Why?

AI coding assistants struggle with large codebases because they can't see the full picture. agentmap provides:

- **Symbol maps** so AI knows what's in large files without reading them entirely
- **Extracted warnings** so AI doesn't miss critical TODOs or safety notes
- **Reading order** so AI starts from the right entry points

## Installation

### From Source

```bash
cargo install --path .
```

### Build Locally

```bash
git clone https://github.com/user/agentmap
cd agentmap
cargo build --release
./target/release/agentmap --help
```

## Usage

### Basic

```bash
# Generate docs for current directory
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
  -o, --output <OUTPUT>        Output directory [default: .agentmap]
  -t, --threshold <THRESHOLD>  Line threshold for "large" files [default: 500]
  -d, --depth <DEPTH>          Max directory depth (0 = unlimited) [default: 0]
      --diff <REF>             Compare against git branch/commit
      --json                   Output JSON to stdout instead of markdown files
  -i, --ignore <IGNORE>        Additional patterns to ignore
  -l, --lang <LANG>            Filter by language
      --no-gitignore           Don't respect .gitignore
      --dry-run                Preview output without writing
  -v, --verbose...             Increase verbosity (-v, -vv, -vvv)
  -q, --quiet                  Suppress all output
  -h, --help                   Print help
  -V, --version                Print version
```

## Example Output

### outline.md

```markdown
## src/analyze/parser.rs (450 lines)

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 15 | fn | parse_symbols | pub |
| 89 | fn | extract_functions | (private) |
| 156 | struct | ParseResult | pub |

### Key Entry Points
- `pub fn parse_symbols(content: &str) -> Vec<Symbol>` (L15)
```

### memory.md

```markdown
## ⚠️ Warnings

### 🔴 `WARNING` (src/auth.rs:42)
> Never store passwords in plain text

## 🔧 Technical Debt

### 🟡 `TODO` (src/api.rs:128)
> Implement rate limiting before production
```

### AGENTS.md

```markdown
## Reading Protocol

**MUST**:
- Read `outline.md` before exploring large files
- Check `memory.md` for warnings and business rules

## Entry Points
- `src/main.rs`
- `src/lib.rs`

## Large Files (Consult outline.md)
| File | Lines |
| ---- | ----- |
| `src/parser.rs` | 892 |
```

## Supported Languages

| Language | Symbol Extraction | Import Graph | Memory Markers |
|----------|-------------------|--------------|----------------|
| Rust | ✅ Functions, structs, enums, traits, impls | ✅ | ✅ |
| Python | ✅ Functions, classes, methods | ✅ | ✅ |
| JavaScript/TypeScript | ✅ Functions, classes, arrow functions | ✅ | ✅ |
| Go | ✅ Functions, structs, interfaces, methods | ✅ | ✅ |
| Swift | ✅ Functions, classes, structs, enums, protocols | ✅ | ✅ |
| Dart | ✅ Functions, classes, mixins, extensions | ✅ | ✅ |
| Ruby | ✅ Methods, classes, modules | ✅ | ✅ |
| C# | ✅ Methods, classes, structs, interfaces | ✅ | ✅ |
| Java | ✅ Methods, classes, interfaces, enums | ✅ | ✅ |

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
1. .agentmap/AGENTS.md - for reading instructions
2. .agentmap/memory.md - for warnings and TODOs
3. .agentmap/outline.md - for large file navigation
```

### GitHub Copilot

Include `.agentmap/` in your workspace context.

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
