# MCP Server

agentlens can run as an [MCP (Model Context Protocol)](https://modelcontextprotocol.io/) server, enabling AI tools like Claude Desktop, Cursor, and others to query and regenerate codebase documentation programmatically.

## Quick Start

```bash
# Using npx (no install required)
npx @agentlens/cli serve --mcp

# Using bunx
bunx @agentlens/cli serve --mcp

# Or if installed globally
agentlens serve --mcp
```

## Available Tools

The MCP server exposes 4 tools:

### `regenerate`

Regenerate agentlens documentation for the codebase.

**Parameters:** None

**Example Response:**
```
Documentation regenerated successfully
```

### `get_module`

Get module documentation by slug (combines MODULE.md, outline.md, memory.md, imports.md).

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `slug` | string | Yes | Module slug (e.g., `src-analyze`, `src-cli`) |

**Example:**
```json
{ "slug": "src-analyze" }
```

**Response:** Combined markdown content from all module files.

### `check_stale`

Check if documentation is stale and needs regeneration.

**Parameters:** None

**Example Response:**
```json
{
  "is_stale": true,
  "stale_modules": ["src-cli"],
  "new_modules": [],
  "removed_modules": []
}
```

### `get_outline`

Get symbol outline for a specific file (functions, structs, classes, etc.).

**Parameters:**

| Name | Type | Required | Description |
|------|------|----------|-------------|
| `file` | string | Yes | Relative file path (e.g., `src/main.rs`) |

**Example:**
```json
{ "file": "src/main.rs" }
```

**Response:**
```markdown
# src/main.rs

| Line | Kind | Name | Visibility |
| ---- | ---- | ---- | ---------- |
| 15 | fn | main | pub |
| 42 | fn | run_analysis | (private) |
```

## Resources

The server also exposes resources:

| URI | Description |
|-----|-------------|
| `agentlens://index` | Read INDEX.md content |

## Client Configuration

### Claude Desktop

Add to `claude_desktop_config.json`:

```json
{
  "mcpServers": {
    "agentlens": {
      "command": "npx",
      "args": ["@agentlens/cli", "serve", "--mcp"],
      "cwd": "/path/to/your/project"
    }
  }
}
```

**Config file locations:**
- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Windows: `%APPDATA%\Claude\claude_desktop_config.json`
- Linux: `~/.config/Claude/claude_desktop_config.json`

### Claude Code

Use the CLI to add the MCP server:

```bash
# Add with user scope (available in all projects)
claude mcp add agentlens --scope user -- npx @agentlens/cli serve --mcp

# Or add with project scope (only this project)
claude mcp add agentlens --scope project -- npx @agentlens/cli serve --mcp
```

**Or edit config directly** at `~/.claude/settings.json`:

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

Verify installation:

```bash
claude mcp list
```

### OpenCode

Add to `opencode.json` (project root) or `~/.config/opencode/config.json` (global):

```json
{
  "$schema": "https://opencode.ai/config.json",
  "mcp": {
    "agentlens": {
      "type": "local",
      "command": ["npx", "@agentlens/cli", "serve", "--mcp"],
      "enabled": true
    }
  }
}
```

See [OpenCode MCP docs](https://opencode.ai/docs/mcp-servers/) for more options.

### Cursor

Add to `.cursor/mcp.json` in your project:

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

### Windsurf

Add to `~/.codeium/windsurf/mcp_config.json`:

```json
{
  "mcpServers": {
    "agentlens": {
      "command": "npx",
      "args": ["@agentlens/cli", "serve", "--mcp"],
      "cwd": "/path/to/your/project"
    }
  }
}
```

### Zed

Add to Zed settings (`~/.config/zed/settings.json`):

```json
{
  "context_servers": {
    "agentlens": {
      "command": {
        "path": "npx",
        "args": ["@agentlens/cli", "serve", "--mcp"]
      }
    }
  }
}
```

### VS Code (with MCP extension)

If using an MCP extension, add to `.vscode/mcp.json`:

```json
{
  "servers": {
    "agentlens": {
      "command": "npx",
      "args": ["@agentlens/cli", "serve", "--mcp"]
    }
  }
}
```

### Generic stdio Client

Any MCP-compatible client can connect via stdio transport:

```bash
npx @agentlens/cli serve --mcp
```

The server communicates via JSON-RPC over stdin/stdout.

## Use Cases

### 1. Query Module Documentation

```
AI: Use get_module with slug "src-analyze" to understand the analysis module.
```

### 2. Check Before Editing

```
AI: Use check_stale to see if docs need regeneration before making changes.
```

### 3. Navigate Large Files

```
AI: Use get_outline for "src/parser.rs" to see what functions are defined.
```

### 4. Keep Docs Fresh

```
AI: Use regenerate after making code changes to update documentation.
```

## Technical Details

- **Transport:** stdio (JSON-RPC over stdin/stdout)
- **Protocol:** MCP (Model Context Protocol)
- **Library:** [rmcp](https://crates.io/crates/rmcp) Rust MCP implementation

## Troubleshooting

### Server not starting

Ensure agentlens is in your PATH:

```bash
which agentlens
```

If not found, reinstall:

```bash
cargo install agentlens
```

### Permission denied

Make sure the binary is executable:

```bash
chmod +x $(which agentlens)
```

### Module not found errors

Run agentlens once to generate docs:

```bash
agentlens
```

Then start the MCP server.
