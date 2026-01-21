<div align="center">

# 🔍 agentlens

**Cho AI assistant khả năng nhìn xuyên thấu codebase của bạn**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![npm](https://img.shields.io/npm/v/@agentlens/cli)](https://www.npmjs.com/package/@agentlens/cli)
[![Homebrew](https://img.shields.io/badge/homebrew-available-blue)](https://github.com/nguyenphutrong/homebrew-tap)

[🇬🇧 English](README.md) · [Bắt Đầu Nhanh](#-bắt-đầu-nhanh) · [Tài Liệu](#-tài-liệu)

</div>

---

## Vấn Đề

AI coding assistants bị **mù** trong codebase lớn. Chúng không thể thấy:
- Có những module nào và chúng liên kết ra sao
- Có gì trong file mà không đọc toàn bộ
- Warnings và TODOs đang ẩn ở đâu
- Cách điều hướng hiệu quả

## Giải Pháp

**agentlens** tạo ra một lớp tài liệu có cấu trúc giúp AI assistants có bản đồ codebase:

```
.agentlens/
├── INDEX.md              # 🗺️  Bảng định hướng toàn cục
├── modules/
│   └── {module}/
│       ├── MODULE.md     # 📦 Tổng quan module
│       ├── outline.md    # 🔎 Bản đồ symbol
│       ├── memory.md     # ⚠️  Warnings & TODOs
│       └── imports.md    # 🔗 Dependencies
└── files/
    └── {file}.md         # 📄 Docs chi tiết (chỉ file phức tạp)
```

**Kết quả:** AI chỉ load những gì cần. Không còn context overflow. Không còn hallucinations về cấu trúc code.

---

## 📊 Tiết Kiệm Token Đã Được Chứng Minh

Benchmark thực tế trên **codebase PHP/Laravel 362K dòng code**:

| Scenario | Tokens | Chi phí (GPT-5.1-codex-mini) |
|----------|--------|------------------------------|
| Đọc toàn bộ source code | ~3,627,260 | $0.91 |
| Đọc toàn bộ AgentLens docs | 129,850 | $0.032 |
| Phân cấp (INDEX + 1 module) | ~25,580 | $0.006 |

**Giảm Token:**
- 📉 **96.4%** ít tokens hơn so với đọc raw source
- 📉 **80.3%** ít tokens hơn với điều hướng phân cấp
- 💰 **$0.006** mỗi lần điều hướng thay vì $0.91

```bash
# Phân tích codebase của bạn
agentlens telemetry summary
```

---

## ⚡ Bắt Đầu Nhanh

### Cài Đặt

```bash
# npm/pnpm/yarn/bun - Khuyến nghị
npx @agentlens/cli            # Chạy không cần cài
npm install -g @agentlens/cli
pnpm add -g @agentlens/cli
yarn global add @agentlens/cli
bun add -g @agentlens/cli

# Homebrew (macOS)
brew install nguyenphutrong/tap/agentlens

# Cargo
cargo install agentlens

# Script cài nhanh
curl -fsSL https://raw.githubusercontent.com/nguyenphutrong/agentlens/main/scripts/install.sh | sh
```

**Cách khác:** Copy prompt này cho AI coding assistant của bạn:

```
Install and configure agentlens by following the instructions at:
https://github.com/nguyenphutrong/agentlens/blob/main/docs/ai-agent-setup.md
```

### Chạy

```bash
# Tạo docs cho thư mục hiện tại
agentlens

# Xong. Kiểm tra .agentlens/INDEX.md
```

### Nói Với AI

Thêm vào instructions của AI (`.cursorrules`, `CLAUDE.md`, v.v.):

```
Trước khi làm việc với codebase này, đọc .agentlens/INDEX.md để điều hướng.
```

---

## ✨ Tính Năng Chính

| Tính năng | Công dụng |
|-----------|-----------|
| **🧠 Docs Phân Cấp** | AI load theo module, không phải toàn bộ codebase |
| **📊 Token Telemetry** | Đo lường và xác minh tiết kiệm token |
| **📦 Tự Động Phát Hiện Module** | Tìm `mod.rs`, `__init__.py`, `index.ts` tự động |
| **🔎 Bản Đồ Symbol** | Biết có gì trong file 1000 dòng mà không đọc hết |
| **⚠️ Memory Markers** | Hiển thị `TODO`, `FIXME`, `WARNING` comments |
| **🔗 Import Graphs** | Cho thấy các module phụ thuộc nhau thế nào |
| **⚡ Cập Nhật Incremental** | Chỉ regenerate modules đã thay đổi |
| **👀 Watch Mode** | Tự động regenerate khi save file |
| **🪝 Git Hooks** | Giữ docs đồng bộ qua các branches |
| **🌐 Remote Repos** | Phân tích GitHub repos trực tiếp |
| **🔌 MCP Server** | Tích hợp native với Claude Desktop & Cursor |
| **🔍 Semantic Search** | Tìm code bằng truy vấn ngôn ngữ tự nhiên |

---

## 📖 Tài Liệu

### Cách Dùng Cơ Bản

```bash
agentlens                    # Tạo docs (hierarchical mode)
agentlens -o docs/ai         # Thư mục output tùy chỉnh
agentlens --dry-run          # Xem trước không ghi file
agentlens -v                 # Output chi tiết
```

### Remote Repositories

```bash
agentlens github.com/vercel/next.js
agentlens --depth 3 github.com/facebook/react
```

### Git Diff Mode

```bash
agentlens --diff main        # Chỉ files thay đổi từ main
agentlens --diff HEAD~5      # So sánh với commit cụ thể
```

### JSON Output

```bash
agentlens --json > analysis.json
agentlens --json | jq '.modules[] | {slug, file_count}'
```

### Watch Mode

```bash
agentlens watch              # Tự động regenerate khi file thay đổi
agentlens watch --debounce 500
```

### Git Hooks

```bash
agentlens hooks install      # Tự động phát hiện Husky/Lefthook/native
agentlens hooks remove       # Gỡ hooks
AGENTLENS_SKIP=1 git commit  # Bỏ qua tạm thời
```

Hỗ trợ: **Husky**, **Lefthook**, **pre-commit**, **native git hooks**

### CI Integration

```bash
agentlens --check            # Exit 1 nếu docs đã stale
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

Tìm kiếm codebase bằng truy vấn ngôn ngữ tự nhiên. Yêu cầu [Ollama](https://ollama.ai/) với model `nomic-embed-text`.

```bash
# Đầu tiên, tạo search index
agentlens index

# Tìm kiếm bằng ngôn ngữ tự nhiên
agentlens search "authentication flow"
agentlens search "error handling" --limit 20
agentlens search "database queries" --hybrid   # kết hợp vector + text search

# Quản lý index
agentlens index status   # Hiển thị thống kê index
agentlens index clear    # Xóa index
```

**Cài đặt Ollama:**
```bash
# Cài Ollama (macOS)
brew install ollama

# Pull embedding model
ollama pull nomic-embed-text

# Khởi động Ollama server
ollama serve
```

---

## 🗂️ Cấu Trúc Output

| Level | File | Mục đích | Kích thước |
|-------|------|----------|------------|
| **L0** | `INDEX.md` | Bảng định hướng toàn cục | O(modules) |
| **L1** | `MODULE.md` | Tóm tắt module & danh sách file | O(files) |
| **L1** | `outline.md` | Bản đồ symbol cho files lớn | O(symbols) |
| **L1** | `memory.md` | Warnings & TODOs | O(markers) |
| **L1** | `imports.md` | Dependencies | O(imports) |
| **L2** | `files/*.md` | Docs chi tiết cho files phức tạp | O(symbols) |

---

## 🌍 Ngôn Ngữ Hỗ Trợ

| Ngôn ngữ | Symbols | Imports | Memory | Modules |
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

agentlens trích xuất các comment patterns sau:

| Pattern | Category | Priority |
|---------|----------|----------|
| `TODO`, `FIXME`, `XXX`, `BUG`, `HACK` | Technical Debt | Medium |
| `WARNING`, `WARN` | Warnings | High |
| `SAFETY`, `INVARIANT` | Safety | High |
| `RULE`, `POLICY` | Business Rules | High |
| `DEPRECATED` | Technical Debt | High |
| `NOTE` | Notes | Low |

---

## ⚙️ Cấu Hình

```bash
agentlens init --config      # Tạo agentlens.toml
```

```toml
output = ".agentlens"
threshold = 500              # Số dòng cho file "lớn"
complex_threshold = 1000     # Số symbols cho L2 docs
ignore = ["*.test.ts", "fixtures/", "__mocks__/"]

[watch]
debounce_ms = 300

[search]
[search.embedder]
provider = "ollama"
model = "nomic-embed-text"
dimensions = 768
# endpoint = "http://localhost:11434"  # Ollama endpoint tùy chỉnh

[search.chunking]
max_tokens = 512
overlap_tokens = 50
strategy = "symbol"          # "symbol" hoặc "window"

[search.search]
hybrid_enabled = true        # Kết hợp vector + text search
hybrid_k = 60.0              # Tham số RRF fusion
default_limit = 10
```

### AI Tool Templates

```bash
agentlens init --templates              # Tất cả templates
agentlens init --templates=cursor       # Chỉ .cursorrules
agentlens init --templates=claude       # Chỉ CLAUDE.md
```

---

## 🤔 Có Nên Commit `.agentlens/`?

| Team Size | Khuyến nghị |
|-----------|-------------|
| **Solo / Nhỏ (1-5)** | ✅ Commit — docs có sẵn khi clone |
| **Vừa (5-15)** | ❌ Ignore — tránh merge conflicts |
| **Lớn (15+)** | ❌ Ignore — dùng CI để validate |
| **Open Source** | ✅ Commit — showcase cho contributors |

Nếu ignore, thêm `.agentlens/` vào `.gitignore` và chạy `agentlens hooks install`.

---

## 🛠️ CLI Reference

```
Usage: agentlens [OPTIONS] [PATH]

Arguments:
  [PATH]  Thư mục đích hoặc GitHub URL [default: .]

Options:
  -o, --output <DIR>         Thư mục output [default: .agentlens]
  -t, --threshold <N>        Ngưỡng file lớn [default: 500]
  -c, --complex-threshold    Ngưỡng L2 docs [default: 30]
  -d, --depth <N>            Max directory depth (0 = unlimited)
      --diff <REF>           So sánh với git ref
      --json                 Output JSON ra stdout
      --check                Kiểm tra docs có stale không
      --force                Force regenerate tất cả modules
  -i, --ignore <PATTERN>     Patterns bổ sung để bỏ qua
  -l, --lang <LANG>          Lọc theo ngôn ngữ
      --no-gitignore         Không tuân theo .gitignore
      --dry-run              Xem trước không ghi file
  -v, --verbose              Tăng mức chi tiết (-v, -vv, -vvv)
  -q, --quiet                Không hiển thị output
  -h, --help                 In help
  -V, --version              In version

Commands:
  watch       Theo dõi changes và regenerate
  hooks       Quản lý git hooks
  init        Khởi tạo cấu hình
  serve       Khởi động MCP server
  telemetry   Phân tích token usage và hiệu quả
  index       Tạo semantic search index
  search      Tìm kiếm codebase bằng ngôn ngữ tự nhiên
  update      Cập nhật lên phiên bản mới nhất
```

### Telemetry Commands

```bash
agentlens telemetry summary          # Phân tích token cho tất cả modules
agentlens telemetry module <SLUG>    # Phân tích module cụ thể
```

Output ví dụ:
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

MIT License — xem [LICENSE](LICENSE)

---

<div align="center">

**Xây dựng cho AI agents. Bởi con người. Tạm thời.**

[GitHub](https://github.com/nguyenphutrong/agentlens) · [npm](https://www.npmjs.com/package/@agentlens/cli) · [Issues](https://github.com/nguyenphutrong/agentlens/issues)

</div>
