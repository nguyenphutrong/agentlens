# agentlens

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

**Chuẩn bị codebase cho AI agents** bằng cách tạo tài liệu có cấu trúc giúp trợ lý AI hiểu và điều hướng code của bạn hiệu quả hơn.

[🇬🇧 English](README.md)

## Công Dụng

agentlens quét codebase và tạo ra **cấu trúc tài liệu phân cấp** theo module:

```
.agentlens/
├── INDEX.md              # L0: Bảng định hướng toàn cục
├── modules/
│   └── {module-slug}/
│       ├── MODULE.md     # L1: Tóm tắt module
│       ├── outline.md    # L1: Bản đồ symbol cho module
│       ├── memory.md     # L1: Warnings/TODOs theo module
│       └── imports.md    # L1: Dependencies cho module
└── files/
    └── {file-slug}.md    # L2: Tài liệu chi tiết cho file phức tạp (tùy chọn)
```

### Content Hierarchy

| Cấp | File | Mục đích | Kích thước |
|-----|------|----------|------------|
| L0 | `INDEX.md` | Bảng định hướng tổng quan theo module | O(modules) |
| L1 | `MODULE.md` | Tóm tắt module, file list, entry points | O(files in module) |
| L1 | `outline.md` | Bản đồ symbol cho file lớn trong module | O(large files) |
| L1 | `memory.md` | Warnings và TODOs theo module | O(markers) |
| L1 | `imports.md` | Phụ thuộc trong module | O(imports) |
| L2 | `files/*.md` | Tài liệu chi tiết cho file phức tạp | O(symbols) |

## Tại Sao Cần?

AI coding assistants gặp khó khăn với codebase lớn vì không thể thấy toàn cảnh. agentlens cung cấp:

- **Điều hướng phân cấp** để AI chỉ load module cần thiết
- **Module detection** để gom file thành các nhóm có ý nghĩa
- **Symbol maps** để biết có gì trong file lớn mà không đọc toàn bộ
- **Scoped context** để docs chỉ chứa thông tin liên quan

## Cài Đặt

### Cài Nhanh (Khuyến nghị)

```bash
curl -fsSL https://raw.githubusercontent.com/nguyenphutrong/agentlens/main/scripts/install.sh | sh
```

### Từ crates.io

```bash
cargo install agentlens
```

### Từ Source

```bash
git clone https://github.com/nguyenphutrong/agentlens
cd agentlens
cargo install --path .
```

### Tải Thủ Công

Tải prebuilt binaries từ [GitHub Releases](https://github.com/nguyenphutrong/agentlens/releases).

## Cách Dùng

### Cơ Bản

```bash
# Tạo docs cho thư mục hiện tại (hierarchical mode - mặc định)
agentlens

# Output ra thư mục tùy chỉnh
agentlens -o docs/ai

# Xem trước mà không ghi file
agentlens --dry-run

# Output chi tiết
agentlens -v
```

### Remote Repositories

```bash
# Phân tích GitHub repo trực tiếp
agentlens github.com/user/repo
agentlens https://github.com/vercel/next.js

# Giới hạn depth cho repo lớn
agentlens --depth 3 github.com/facebook/react
```

### Git Diff Mode

```bash
# Chỉ show các file thay đổi từ branch
agentlens --diff main

# So sánh với commit cụ thể
agentlens --diff HEAD~5
```

### JSON Output

```bash
# Output analysis dưới dạng JSON (cho tooling integration)
agentlens --json > analysis.json

# Kết hợp với flags khác
agentlens --json --depth 2 github.com/user/repo
```

### Options

```
Usage: agentlens [OPTIONS] [PATH]

Arguments:
  [PATH]  Thư mục đích hoặc GitHub URL [default: .]

Options:
  -o, --output <OUTPUT>              Thư mục output [default: .agentlens]
  -t, --threshold <THRESHOLD>        Ngưỡng số dòng cho file "lớn" [default: 500]
  -c, --complex-threshold <COMPLEX>  Ngưỡng symbol cho L2 file docs [default: 30]
  -d, --depth <DEPTH>                Max directory depth (0 = unlimited)
      --diff <REF>                   So sánh với git branch/commit
      --json                         Output JSON ra stdout
      --check                        Kiểm tra docs có stale không (exit 1 nếu cần regenerate)
      --config <FILE>                Đường dẫn config file
      --force                        Force regenerate tất cả modules (bỏ qua cache)
  -i, --ignore <IGNORE>              Patterns bổ sung để bỏ qua
  -l, --lang <LANG>                  Lọc theo ngôn ngữ
      --no-gitignore                 Không tuân theo .gitignore
      --dry-run                      Xem trước mà không ghi file
  -v, --verbose...                   Tăng mức chi tiết (-v, -vv, -vvv)
  -q, --quiet                        Không hiển thị output
  -h, --help                         In help
  -V, --version                      In version

Commands:
  watch   Theo dõi file changes và tự động regenerate docs
  hooks   Quản lý git hooks cho auto-regeneration
  init    Khởi tạo cấu hình agentlens
  update  Cập nhật agentlens lên phiên bản mới nhất
```

## Watch Mode

Giữ docs luôn đồng bộ trong quá trình development:

```bash
# Bắt đầu theo dõi changes (regenerate khi save file)
agentlens watch

# Tuỳ chỉnh debounce delay (mặc định: 300ms)
agentlens watch --debounce 500
```

Watch mode tận dụng hệ thống manifest incremental, nên chỉ modules thay đổi được regenerate.

## Git Hooks

Tự động regenerate docs tại các git events quan trọng:

```bash
# Cài hooks (pre-commit, post-checkout, post-merge)
agentlens hooks install

# Gỡ hooks
agentlens hooks remove

# Bỏ qua hooks tạm thời
AGENTLENS_SKIP=1 git commit -m "quick fix"
```

Các hooks được cài:
- **pre-commit**: Regenerate docs và stage `.agentlens/`
- **post-checkout**: Regenerate sau khi đổi branch (chạy nền)
- **post-merge**: Regenerate sau pull/merge (chạy nền)

## Configuration File

Tạo `agentlens.toml` cho cài đặt riêng của project:

```bash
# Tạo config file mặc định
agentlens init --config

# Dùng config tùy chỉnh
agentlens --config custom.toml
```

Ví dụ `agentlens.toml`:

```toml
output = ".agentlens"
threshold = 500
complex_threshold = 1000
ignore = ["*.test.ts", "fixtures/", "__mocks__/"]

[watch]
debounce_ms = 300
```

CLI flags ghi đè giá trị config file.

## MCP Server

agentlens có thể chạy như MCP server cho các AI tools như Claude Desktop và Cursor:

```bash
# Sử dụng npx (không cần cài đặt)
npx @agentlens/cli serve --mcp

# Hoặc nếu đã cài global
agentlens serve --mcp
```

**Các tools có sẵn:**

| Tool | Mô tả |
|------|-------|
| `regenerate` | Regenerate tài liệu |
| `get_module` | Lấy docs module theo slug |
| `check_stale` | Kiểm tra docs có cần cập nhật |
| `get_outline` | Lấy symbol outline cho file |

**Ví dụ MCP config (Claude Desktop, Cursor, etc.):**

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

Xem [MCP Server Documentation](docs/mcp-server.vi.md) để biết hướng dẫn cài đặt và tích hợp.

## CI Integration

Validate docs freshness trong CI pipelines:

```bash
# Kiểm tra docs có stale không (exit 0 = fresh, exit 1 = stale)
agentlens --check

# Kết hợp với diff mode
agentlens --check --diff main
```

Ví dụ GitHub Actions workflow:

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

agentlens tự động phát hiện module boundary dựa vào quy ước theo ngôn ngữ:

| Ngôn ngữ | Boundary rõ ràng | Ví dụ |
|----------|------------------|-------|
| Rust | `mod.rs`, `lib.rs` | `src/analyze/mod.rs` → module `src-analyze` |
| Python | `__init__.py` | `src/utils/__init__.py` → module `src-utils` |
| JavaScript/TypeScript | `index.{js,ts,tsx}` | `src/components/index.ts` → module `src-components` |
| Any | 5+ source files trong thư mục | `src/helpers/` có 5+ files → implicit module |

### Module Slug Naming

Đường dẫn thư mục được chuyển thành slug bằng dấu gạch ngang:
- `src/analyze/lang` → `src-analyze-lang`
- `lib/utils` → `lib-utils`

## Ví Dụ Output

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

agentlens trích xuất các comment patterns sau:

| Pattern | Category | Priority |
|---------|----------|----------|
| `TODO`, `FIXME`, `XXX`, `BUG`, `HACK` | Technical Debt | Medium |
| `WARNING`, `WARN` | Warnings | High |
| `SAFETY`, `INVARIANT` | Safety | High |
| `RULE`, `POLICY` | Business Rules | High |
| `DEPRECATED` | Technical Debt | High |
| `NOTE` | Notes | Low |

## Tích Hợp với AI Tools

### Claude Code / Cursor

Thêm vào AI instructions của project:

```
Trước khi làm việc với codebase này, đọc:
1. .agentlens/INDEX.md - tổng quan và điều hướng module
2. Navigate to module's MODULE.md để biết chi tiết
3. Kiểm tra module's memory.md trước khi edit
4. Consult module's outline.md để điều hướng file lớn
```

### GitHub Copilot

Include `.agentlens/` trong workspace context.

### JSON Integration

Cho programmatic access:

```bash
agentlens --json | jq '.modules[] | {slug, file_count, warning_count}'
```

JSON output gồm:
- `modules[]` - Array module metadata (slug, path, file_count, warning_count, symbol_count, is_hub)
- `files[]` - Tất cả scanned files và metadata
- `memory[]` - Tất cả memory markers và locations
- `entry_points[]` - Detected entry points
- `hub_files[]` - Files được import bởi 3+ others

## Development

```bash
# Chạy tests
cargo test

# Chạy với verbose output
cargo run -- -vv .

# Kiểm tra issues
cargo clippy
```

## License

MIT License - xem [LICENSE](LICENSE) để biết chi tiết.
