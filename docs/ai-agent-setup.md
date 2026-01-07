# AI Agent Setup Guide

Instructions for AI coding assistants to install and use agentmap.

## Quick Install (Choose One)

```bash
# Option 1: npx (no install required)
npx agentmap-cli

# Option 2: Homebrew (macOS/Linux)
brew install nguyenphutrong/tap/agentmap

# Option 3: npm global
npm install -g agentmap-cli

# Option 4: Cargo
cargo install agentmap

# Option 5: Direct download
curl -fsSL https://raw.githubusercontent.com/nguyenphutrong/agentmap/main/scripts/install.sh | sh
```

## Generate Documentation

```bash
# Generate docs for current project
agentmap

# Output: .agentmap/ directory with hierarchical docs
```

## Reading Protocol

After generation, navigate the documentation hierarchy:

```
1. Start with .agentmap/INDEX.md
   → Project overview, module list, entry points

2. Go to relevant module: .agentmap/modules/{name}/MODULE.md
   → File list, module structure, child modules

3. Check module docs as needed:
   → outline.md  - Symbol maps for large files
   → memory.md   - Warnings, TODOs, technical debt
   → imports.md  - Dependencies between files
```

## MCP Server (Recommended)

Run agentmap as an MCP server for real-time codebase queries:

```bash
npx agentmap-cli serve --mcp
```

### MCP Config

Add to your AI tool's MCP configuration:

```json
{
  "mcpServers": {
    "agentmap": {
      "command": "npx",
      "args": ["agentmap-cli", "serve", "--mcp"]
    }
  }
}
```

### Available MCP Tools

| Tool | Description |
|------|-------------|
| `regenerate` | Regenerate documentation |
| `get_module` | Get module docs by slug |
| `check_stale` | Check if docs need update |
| `get_outline` | Get symbol outline for a file |

## Project Integration

### Add to AI Instructions

Include in your project's `.cursorrules`, `CLAUDE.md`, or `AGENTS.md`:

```markdown
## Codebase Navigation

Before working on this codebase:
1. Read .agentmap/INDEX.md for project overview
2. Navigate to relevant module's MODULE.md
3. Check memory.md for warnings before editing
4. Use outline.md for large file navigation
```

### Generate Templates

```bash
# Generate AI tool configuration templates
agentmap init --templates

# Generates:
# - .cursorrules (Cursor IDE)
# - CLAUDE.md (Claude Code)
# - AGENTS.md (OpenCode)
```

## Keep Docs Fresh

### Option A: Git Hooks (Automatic)

```bash
agentmap hooks install
```

Docs regenerate on commit, checkout, and merge.

### Option B: Watch Mode (Development)

```bash
agentmap watch
```

Docs regenerate on file save.

### Option C: Manual

```bash
agentmap --check  # Check if stale
agentmap          # Regenerate
```

## Troubleshooting

### Command not found

```bash
# Verify installation
which agentmap || npx agentmap-cli --version

# Reinstall if needed
npm install -g agentmap-cli
```

### Docs seem stale

```bash
agentmap --force  # Force full regeneration
```

### Large repository slow

```bash
agentmap --depth 3  # Limit directory depth
agentmap -i "test/,fixtures/"  # Ignore patterns
```
