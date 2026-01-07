# agentmap-cli

CLI tool to prepare codebases for AI agents by generating hierarchical documentation.

## Installation

```bash
# Using npx (no install required)
npx agentmap-cli

# Or install globally
npm install -g agentmap-cli
```

## Usage

```bash
# Generate docs for current directory
npx agentmap-cli

# Start MCP server for AI tools
npx agentmap-cli serve --mcp
```

## MCP Server

Use agentmap as an MCP server with Claude Desktop, Cursor, OpenCode, and other AI tools:

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

## Documentation

- [Full Documentation](https://github.com/nguyenphutrong/agentmap)
- [MCP Server Setup](https://github.com/nguyenphutrong/agentmap/blob/main/docs/mcp-server.md)

## License

MIT
