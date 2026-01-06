# agentmap

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)

**Chuẩn bị codebase cho AI agents** bằng cách tạo tài liệu có cấu trúc giúp trợ lý AI hiểu và điều hướng code của bạn hiệu quả hơn.

[🇬🇧 English](README.md)

## Công Dụng

agentmap quét codebase và tạo ra 3 file:

| File | Mục đích |
|------|----------|
| `outline.md` | Bản đồ symbol cho file lớn (functions, classes, structs với số dòng) |
| `memory.md` | Các markers được trích xuất (TODO, FIXME, WARNING, SAFETY, business rules) |
| `AGENTS.md` | Hướng dẫn đọc code cho AI agents (entry points, critical files, rules) |

## Tại Sao Cần?

AI coding assistants gặp khó khăn với codebase lớn vì không thể thấy toàn cảnh. agentmap cung cấp:

- **Bản đồ symbol** để AI biết có gì trong file lớn mà không cần đọc toàn bộ
- **Warnings được trích xuất** để AI không bỏ sót TODO hoặc safety notes quan trọng
- **Thứ tự đọc** để AI bắt đầu từ đúng entry points

## Cài Đặt

### Từ Source

```bash
cargo install --path .
```

### Build Local

```bash
git clone https://github.com/nguyenphutrong/agentmap
cd agentmap
cargo build --release
./target/release/agentmap --help
```

## Cách Dùng

### Cơ Bản

```bash
# Tạo docs cho thư mục hiện tại
agentmap

# Output ra thư mục tùy chỉnh
agentmap -o docs/ai

# Xem trước mà không ghi file
agentmap --dry-run

# Output chi tiết
agentmap -v
```

### Các Options

```
Usage: agentmap [OPTIONS] [PATH]

Arguments:
  [PATH]  Thư mục đích [default: .]

Options:
  -o, --output <OUTPUT>        Thư mục output [default: .agentmap]
  -t, --threshold <THRESHOLD>  Ngưỡng số dòng cho file "lớn" [default: 500]
  -i, --ignore <IGNORE>        Patterns bổ sung để bỏ qua
  -l, --lang <LANG>            Lọc theo ngôn ngữ (rust, python, javascript, go)
      --no-gitignore           Không tuân theo .gitignore
      --dry-run                Xem trước mà không ghi file
  -v, --verbose...             Tăng mức chi tiết (-v, -vv, -vvv)
  -q, --quiet                  Không hiển thị output
  -h, --help                   In help
  -V, --version                In version
```

## Ví Dụ Output

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
> Không bao giờ lưu passwords dạng plain text

## 🔧 Technical Debt

### 🟡 `TODO` (src/api.rs:128)
> Implement rate limiting trước khi lên production
```

### AGENTS.md

```markdown
## Reading Protocol

**MUST**:
- Đọc `outline.md` trước khi khám phá file lớn
- Kiểm tra `memory.md` để biết warnings và business rules

## Entry Points
- `src/main.rs`
- `src/lib.rs`

## Large Files (Tham khảo outline.md)
| File | Lines |
| ---- | ----- |
| `src/parser.rs` | 892 |
```

## Ngôn Ngữ Hỗ Trợ

| Ngôn ngữ | Symbol Extraction | Memory Markers |
|----------|-------------------|----------------|
| Rust | ✅ Functions, structs, enums, traits, impls | ✅ |
| Python | ✅ Functions, classes, methods | ✅ |
| JavaScript/TypeScript | ✅ Functions, classes, arrow functions | ✅ |
| Go | ✅ Functions, structs, interfaces, methods | ✅ |

## Memory Markers

agentmap trích xuất các comment patterns sau:

| Pattern | Danh mục | Độ ưu tiên |
|---------|----------|------------|
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
1. .agentmap/AGENTS.md - hướng dẫn đọc code
2. .agentmap/memory.md - warnings và TODOs
3. .agentmap/outline.md - điều hướng file lớn
```

### GitHub Copilot

Include `.agentmap/` trong workspace context.

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
