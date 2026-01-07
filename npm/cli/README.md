# @agentlens/cli

CLI tool to prepare codebases for AI agents by generating hierarchical documentation.

## Installation

```bash
# Using npx (no install required)
npx @agentlens/cli

# Or install globally
npm install -g @agentlens/cli
```

## Usage

```bash
# Generate docs for current directory
npx @agentlens/cli

# Start MCP server for AI tools
npx @agentlens/cli serve --mcp
```

## MCP Server

Use agentlens as an MCP server with Claude Desktop, Cursor, OpenCode, and other AI tools:

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

## Documentation

- [Full Documentation](https://github.com/nguyenphutrong/agentlens)
- [MCP Server Setup](https://github.com/nguyenphutrong/agentlens/blob/main/docs/mcp-server.md)

## License

MIT
