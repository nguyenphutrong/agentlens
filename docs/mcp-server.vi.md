# MCP Server

agentlens có thể chạy như một [MCP (Model Context Protocol)](https://modelcontextprotocol.io/) server, cho phép các AI tools như Claude Desktop, Cursor và các công cụ khác truy vấn và regenerate tài liệu codebase theo chương trình.

## Bắt Đầu Nhanh

```bash
# Sử dụng npx (không cần cài đặt)
npx @agentlens/cli serve --mcp

# Sử dụng bunx
bunx @agentlens/cli serve --mcp

# Hoặc nếu đã cài global
agentlens serve --mcp
```

## Các Tools Có Sẵn

MCP server cung cấp 4 tools:

### `regenerate`

Regenerate tài liệu agentlens cho codebase.

**Parameters:** Không có

**Ví dụ Response:**
```
Documentation regenerated successfully
```

### `get_module`

Lấy tài liệu module theo slug (kết hợp MODULE.md, outline.md, memory.md, imports.md).

**Parameters:**

| Tên | Kiểu | Bắt buộc | Mô tả |
|-----|------|----------|-------|
| `slug` | string | Có | Module slug (vd: `src-analyze`, `src-cli`) |

**Ví dụ:**
```json
{ "slug": "src-analyze" }
```

**Response:** Nội dung markdown kết hợp từ tất cả các file trong module.

### `check_stale`

Kiểm tra xem tài liệu có stale và cần regenerate không.

**Parameters:** Không có

**Ví dụ Response:**
```json
{
  "is_stale": true,
  "stale_modules": ["src-cli"],
  "new_modules": [],
  "removed_modules": []
}
```

### `get_outline`

Lấy symbol outline cho một file cụ thể (functions, structs, classes, v.v.).

**Parameters:**

| Tên | Kiểu | Bắt buộc | Mô tả |
|-----|------|----------|-------|
| `file` | string | Có | Đường dẫn file tương đối (vd: `src/main.rs`) |

**Ví dụ:**
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

Server cũng cung cấp resources:

| URI | Mô tả |
|-----|-------|
| `agentlens://index` | Đọc nội dung INDEX.md |

## Cấu Hình Client

### Claude Desktop

Thêm vào `claude_desktop_config.json`:

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

**Vị trí config file:**
- macOS: `~/Library/Application Support/Claude/claude_desktop_config.json`
- Windows: `%APPDATA%\Claude\claude_desktop_config.json`
- Linux: `~/.config/Claude/claude_desktop_config.json`

### Claude Code

Dùng CLI để thêm MCP server:

```bash
# Thêm với user scope (có sẵn trong mọi project)
claude mcp add agentlens --scope user -- npx @agentlens/cli serve --mcp

# Hoặc thêm với project scope (chỉ project này)
claude mcp add agentlens --scope project -- npx @agentlens/cli serve --mcp
```

**Hoặc sửa config trực tiếp** tại `~/.claude/settings.json`:

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

Kiểm tra cài đặt:

```bash
claude mcp list
```

### OpenCode

Thêm vào `opencode.json` (project root) hoặc `~/.config/opencode/config.json` (global):

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

Xem [OpenCode MCP docs](https://opencode.ai/docs/mcp-servers/) để biết thêm options.

### Cursor

Thêm vào `.cursor/mcp.json` trong project:

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

Thêm vào `~/.codeium/windsurf/mcp_config.json`:

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

Thêm vào Zed settings (`~/.config/zed/settings.json`):

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

### VS Code (với MCP extension)

Nếu dùng MCP extension, thêm vào `.vscode/mcp.json`:

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

Bất kỳ MCP-compatible client nào đều có thể kết nối qua stdio transport:

```bash
npx @agentlens/cli serve --mcp
```

Server giao tiếp qua JSON-RPC qua stdin/stdout.

## Các Use Cases

### 1. Truy Vấn Tài Liệu Module

```
AI: Dùng get_module với slug "src-analyze" để hiểu module analysis.
```

### 2. Kiểm Tra Trước Khi Sửa

```
AI: Dùng check_stale để xem docs có cần regenerate trước khi thay đổi code không.
```

### 3. Điều Hướng File Lớn

```
AI: Dùng get_outline cho "src/parser.rs" để xem các functions được định nghĩa.
```

### 4. Giữ Docs Cập Nhật

```
AI: Dùng regenerate sau khi thay đổi code để cập nhật tài liệu.
```

## Chi Tiết Kỹ Thuật

- **Transport:** stdio (JSON-RPC qua stdin/stdout)
- **Protocol:** MCP (Model Context Protocol)
- **Library:** [rmcp](https://crates.io/crates/rmcp) - Rust MCP implementation

## Xử Lý Sự Cố

### Server không khởi động được

Đảm bảo agentlens nằm trong PATH:

```bash
which agentlens
```

Nếu không tìm thấy, cài lại:

```bash
cargo install agentlens
```

### Permission denied

Đảm bảo binary có quyền thực thi:

```bash
chmod +x $(which agentlens)
```

### Module not found errors

Chạy agentlens một lần để tạo docs:

```bash
agentlens
```

Sau đó khởi động MCP server.
