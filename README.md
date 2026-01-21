<div align="center">

# 🔍 agentlens

**Give your AI assistant X-ray vision into your codebase**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![npm](https://img.shields.io/npm/v/@agentlens/cli)](https://www.npmjs.com/package/@agentlens/cli)
[![Homebrew](https://img.shields.io/badge/homebrew-available-blue)](https://github.com/nguyenphutrong/homebrew-tap)

[🇻🇳 Tiếng Việt](README.vi.md) · [Quick Start](#-quick-start) · [Documentation](#-documentation)

</div>

---

## The Problem

AI coding assistants are **blind** in large codebases. They can't see:
- Which modules exist and how they connect
- What symbols are in files without reading them entirely  
- Where the warnings and TODOs are hiding
- How to navigate efficiently

## The Solution

**agentlens** generates a structured documentation layer that gives AI assistants a map of your codebase:

```
.agentlens/
├── INDEX.md              # 🗺️  Global routing table
├── modules/
│   └── {module}/
│       ├── MODULE.md     # 📦 Module overview
│       ├── outline.md    # 🔎 Symbol maps
│       ├── memory.md     # ⚠️  Warnings & TODOs
│       └── imports.md    # 🔗 Dependencies
└── files/
    └── {file}.md         # 📄 Deep docs (complex files only)
```

**Result:** AI loads only what it needs. No more context overflow. No more hallucinations about code structure.

---

## 📊 Proven Token Savings

Real-world benchmark on a **362K lines PHP/Laravel codebase**:

| Scenario | Tokens | Cost (GPT-5.1-codex-mini) |
|----------|--------|---------------------------|
| Read entire source code | ~3,627,260 | $0.91 |
| Read all AgentLens docs | 129,850 | $0.032 |
| Hierarchical (INDEX + 1 module) | ~25,580 | $0.006 |

**Token Reduction:**
- 📉 **96.4%** less tokens vs reading raw source
- 📉 **80.3%** less tokens with hierarchical navigation
- 💰 **$0.006** per navigation instead of $0.91

```bash
# Analyze your own codebase
agentlens telemetry summary
```

---

## ⚡ Quick Start

### Install

```bash
# npm/pnpm/yarn/bun - Recommended
npx @agentlens/cli            # Run without install
npm install -g @agentlens/cli
pnpm add -g @agentlens/cli
yarn global add @agentlens/cli
bun add -g @agentlens/cli

# Homebrew (macOS)
brew install nguyenphutrong/tap/agentlens

# Cargo
cargo install agentlens

# Quick install script
curl -fsSL https://raw.githubusercontent.com/nguyenphutrong/agentlens/main/scripts/install.sh | sh
```

**Alternative:** Copy this prompt to your AI coding assistant:

```
Install and configure agentlens by following the instructions at:
https://github.com/nguyenphutrong/agentlens/blob/main/docs/ai-agent-setup.md
```

### Run

```bash
# Generate docs for current directory
agentlens

# That's it. Check .agentlens/INDEX.md
```

### Tell Your AI

Add to your AI's instructions (`.cursorrules`, `CLAUDE.md`, etc.):

```
Before working on this codebase, read .agentlens/INDEX.md for navigation.
```

---

## ✨ Key Features

| Feature | What it does |
|---------|--------------|
| **🧠 Hierarchical Docs** | AI loads module-by-module, not entire codebase |
| **📊 Token Telemetry** | Measure and verify token savings |
| **📦 Auto Module Detection** | Finds `mod.rs`, `__init__.py`, `index.ts` automatically |
| **🔎 Symbol Maps** | Know what's in 1000-line files without reading them |
| **⚠️ Memory Markers** | Surfaces `TODO`, `FIXME`, `WARNING` comments |
| **🔗 Import Graphs** | Shows how modules depend on each other |
| **⚡ Incremental Updates** | Only regenerates changed modules |
| **👀 Watch Mode** | Auto-regenerate on file save |
| **🪝 Git Hooks** | Keep docs synced across branches |
| **🌐 Remote Repos** | Analyze GitHub repos directly |
| **🔌 MCP Server** | Native integration with Claude Desktop & Cursor |
| **🔍 Semantic Search** | Find code using natural language queries |

---

## 📖 Documentation

### Basic Usage

```bash
agentlens                    # Generate docs (hierarchical mode)
agentlens -o docs/ai         # Custom output directory
agentlens --dry-run          # Preview without writing
agentlens -v                 # Verbose output
```

### Remote Repositories

```bash
agentlens github.com/vercel/next.js
agentlens --depth 3 github.com/facebook/react
```

### Git Diff Mode

```bash
agentlens --diff main        # Only changed files since main
agentlens --diff HEAD~5      # Compare against specific commit
```

### JSON Output

```bash
agentlens --json > analysis.json
agentlens --json | jq '.modules[] | {slug, file_count}'
```

### Watch Mode

```bash
agentlens watch              # Auto-regenerate on file changes
agentlens watch --debounce 500
```

### Git Hooks

```bash
agentlens hooks install      # Auto-detects Husky/Lefthook/native
agentlens hooks remove       # Remove hooks
AGENTLENS_SKIP=1 git commit  # Skip temporarily
```

Supported: **Husky**, **Lefthook**, **pre-commit**, **native git hooks**

### CI Integration

```bash
agentlens --check            # Exit 1 if docs are stale
```

```yaml
# .github/workflows/docs.yml
- name: Check docs freshness
  run: agentlens --check
```

### MCP Server

```bash
npx @agentlens/cli serve --mcp
```

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

Tools: `regenerate`, `get_module`, `check_stale`, `get_outline`, `semantic_search`

### Semantic Search

Search your codebase using natural language queries. Requires [Ollama](https://ollama.ai/) with `nomic-embed-text` model.

```bash
# First, create the search index
agentlens index

# Search with natural language
agentlens search "authentication flow"
agentlens search "error handling" --limit 20
agentlens search "database queries" --hybrid   # combines vector + text search

# Manage the index
agentlens index status   # Show index statistics
agentlens index clear    # Remove the index
```

**Setup Ollama:**
```bash
# Install Ollama (macOS)
brew install ollama

# Pull the embedding model
ollama pull nomic-embed-text

# Start Ollama server
ollama serve
```

### Agent Skills

Install agentlens skill for AI coding tools (Claude Code, OpenCode, Codex CLI):

```bash
agentlens skills install           # Auto-detect and install
agentlens skills install --claude  # Claude Code only
agentlens skills install --opencode # OpenCode only
agentlens skills install --codex   # Codex CLI only
agentlens skills install --all     # All supported tools
agentlens skills list              # Show installed skills
agentlens skills remove            # Remove skills
```

The skill teaches AI agents how to navigate codebases using agentlens documentation.

---

## 🗂️ Output Structure

| Level | File | Purpose | Size |
|-------|------|---------|------|
| **L0** | `INDEX.md` | Global routing table | O(modules) |
| **L1** | `MODULE.md` | Module summary & file list | O(files) |
| **L1** | `outline.md` | Symbol maps for large files | O(symbols) |
| **L1** | `memory.md` | Warnings & TODOs | O(markers) |
| **L1** | `imports.md` | Dependencies | O(imports) |
| **L2** | `files/*.md` | Deep docs for complex files | O(symbols) |

---

## 🌍 Language Support

| Language | Symbols | Imports | Memory | Modules |
|----------|---------|---------|--------|---------|
| **Rust** | ✅ fn, struct, enum, trait, impl | ✅ | ✅ | `mod.rs` |
| **Python** | ✅ def, class | ✅ | ✅ | `__init__.py` |
| **TypeScript/JS** | ✅ function, class, arrow | ✅ | ✅ | `index.{ts,js}` |
| **PHP** | ✅ function, class, method | ✅ | ✅ | implicit |
| **Go** | ✅ func, struct, interface | ✅ | ✅ | implicit |
| **Swift** | ✅ func, class, struct, enum, protocol | ✅ | ✅ | implicit |
| **Dart** | ✅ function, class, mixin | ✅ | ✅ | implicit |
| **Ruby** | ✅ def, class, module | ✅ | ✅ | implicit |
| **C** | ✅ function, struct | ✅ | ✅ | implicit |
| **C++** | ✅ function, class, struct | ✅ | ✅ | implicit |
| **C#** | ✅ method, class, struct, interface | ✅ | ✅ | implicit |
| **Java** | ✅ method, class, interface, enum | ✅ | ✅ | implicit |

---

## 📝 Memory Markers

agentlens extracts these comment patterns:

| Pattern | Category | Priority |
|---------|----------|----------|
| `TODO`, `FIXME`, `XXX`, `BUG`, `HACK` | Technical Debt | Medium |
| `WARNING`, `WARN` | Warnings | High |
| `SAFETY`, `INVARIANT` | Safety | High |
| `RULE`, `POLICY` | Business Rules | High |
| `DEPRECATED` | Technical Debt | High |
| `NOTE` | Notes | Low |

---

## ⚙️ Configuration

```bash
agentlens init --config      # Generate agentlens.toml
```

```toml
output = ".agentlens"
threshold = 500              # Lines for "large" file
complex_threshold = 1000     # Symbols for L2 docs
ignore = ["*.test.ts", "fixtures/", "__mocks__/"]

[watch]
debounce_ms = 300

[search]
[search.embedder]
provider = "ollama"
model = "nomic-embed-text"
dimensions = 768
# endpoint = "http://localhost:11434"  # Custom Ollama endpoint

[search.chunking]
max_tokens = 512
overlap_tokens = 50
strategy = "symbol"          # "symbol" or "window"

[search.search]
hybrid_enabled = true        # Combine vector + text search
hybrid_k = 60.0              # RRF fusion parameter
default_limit = 10
```

### AI Tool Templates

```bash
agentlens init --templates              # All templates
agentlens init --templates=cursor       # .cursorrules only
agentlens init --templates=claude       # CLAUDE.md only
```

---

## 🤔 Should I Commit `.agentlens/`?

| Team Size | Recommendation |
|-----------|----------------|
| **Solo / Small (1-5)** | ✅ Commit — docs available on clone |
| **Medium (5-15)** | ❌ Ignore — avoid merge conflicts |
| **Large (15+)** | ❌ Ignore — use CI to validate |
| **Open Source** | ✅ Commit — showcase for contributors |

If ignoring, add `.agentlens/` to `.gitignore` and run `agentlens hooks install`.

---

## 🛠️ CLI Reference

```
Usage: agentlens [OPTIONS] [PATH]

Arguments:
  [PATH]  Target directory or GitHub URL [default: .]

Options:
  -o, --output <DIR>         Output directory [default: .agentlens]
  -t, --threshold <N>        Large file threshold [default: 500]
  -c, --complex-threshold    L2 docs threshold [default: 30]
  -d, --depth <N>            Max directory depth (0 = unlimited)
      --diff <REF>           Compare against git ref
      --json                 Output JSON to stdout
      --check                Check if docs are stale
      --force                Force regenerate all modules
  -i, --ignore <PATTERN>     Additional ignore patterns
  -l, --lang <LANG>          Filter by language
      --no-gitignore         Don't respect .gitignore
      --dry-run              Preview without writing
  -v, --verbose              Increase verbosity (-v, -vv, -vvv)
  -q, --quiet                Suppress output
  -h, --help                 Print help
  -V, --version              Print version

Commands:
  watch       Watch for changes and regenerate
  hooks       Manage git hooks
  init        Initialize configuration
  serve       Start MCP server
  skills      Manage agent skills for AI tools
  telemetry   Analyze token usage and efficiency
  index       Build semantic search index
  search      Search codebase with natural language
  update      Update to latest version
```

### Telemetry Commands

```bash
agentlens telemetry summary          # Token analysis for all modules
agentlens telemetry module <SLUG>    # Analyze specific module
```

Example output:
```
📊 Token Analysis: All Modules
═══════════════════════════════════════════════════
| Module | Tokens | Bytes |
|--------|--------|-------|
| app-Models | 23806 | 76021 |
| database-migrations | 15858 | 51106 |
| ... | ... | ... |
|--------|--------|-------|
| **TOTAL (64 modules)** | **128076** | **411225** |

📈 Estimated Cost (GPT-5.1-codex-mini @ $0.25/1M tokens):
  Full codebase read: $0.032
  Hierarchical (INDEX + 1 module): $0.006
  Savings: 80.3%
```

---

## 📄 License

MIT License — see [LICENSE](LICENSE)

---

<div align="center">

**Built for AI agents. By humans. For now.**

[GitHub](https://github.com/nguyenphutrong/agentlens) · [npm](https://www.npmjs.com/package/@agentlens/cli) · [Issues](https://github.com/nguyenphutrong/agentlens/issues)

</div>
