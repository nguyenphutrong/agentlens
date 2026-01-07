# AI Agent Setup Guide

## Step 1: Install

Run ONE of these commands:

```bash
npx @agentlens/cli --version          # Preferred (no install)
brew install nguyenphutrong/tap/agentlens  # macOS
npm install -g @agentlens/cli         # npm global
cargo install agentlens               # Rust
```

Verify: `agentlens --version` should output version number.

## Step 2: Generate Docs

```bash
cd <project-root>
agentlens
```

Output: `.agentlens/` directory created.

## Step 3: Read the Docs

Navigate in this order:

1. `.agentlens/INDEX.md` → Project overview, module list
2. `.agentlens/modules/{name}/MODULE.md` → Module details
3. `.agentlens/modules/{name}/outline.md` → Symbol maps (large files)
4. `.agentlens/modules/{name}/memory.md` → Warnings, TODOs

## Step 4: Keep Fresh (Optional)

Choose one:

```bash
agentlens hooks install  # Auto-regenerate on git events
agentlens watch          # Auto-regenerate on file save
agentlens --check        # Check if stale (CI)
```

## MCP Server (Optional)

For real-time queries, add to MCP config:

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

Tools: `regenerate`, `get_module`, `check_stale`, `get_outline`

## Add to Project AI Instructions

Run once to generate templates:

```bash
agentlens init --templates
```

Creates: `.cursorrules`, `CLAUDE.md`, `AGENTS.md`

## Troubleshooting

| Problem | Solution |
|---------|----------|
| Command not found | `npm install -g @agentlens/cli` |
| Docs stale | `agentlens --force` |
| Slow on large repo | `agentlens --depth 3 -i "test/,node_modules/"` |
