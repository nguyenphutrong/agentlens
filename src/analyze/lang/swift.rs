use crate::analyze::lang::LanguageParser;
use crate::types::{Symbol, SymbolKind, Visibility};
use once_cell::sync::Lazy;
use regex::Regex;

pub struct SwiftParser;

// (public|internal|fileprivate|private|open)? class ClassName: Parent {
static CLASS_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^(?:@\w+\s+)*(public\s+|internal\s+|fileprivate\s+|private\s+|open\s+)?(?:final\s+)?class\s+(\w+)(?:\s*:\s*[^{]+)?\s*\{")
        .unwrap()
});

// struct StructName: Protocol {
static STRUCT_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^(?:@\w+\s+)*(public\s+|internal\s+|fileprivate\s+|private\s+)?struct\s+(\w+)(?:\s*:\s*[^{]+)?\s*\{")
        .unwrap()
});

// enum EnumName: RawType {
static ENUM_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^(?:@\w+\s+)*(public\s+|internal\s+|fileprivate\s+|private\s+)?enum\s+(\w+)(?:\s*:\s*[^{]+)?\s*\{")
        .unwrap()
});

// protocol ProtocolName {
static PROTOCOL_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^(?:@\w+\s+)*(public\s+|internal\s+|fileprivate\s+|private\s+)?protocol\s+(\w+)(?:\s*:\s*[^{]+)?\s*\{")
        .unwrap()
});

// extension TypeName: Protocol {
static EXTENSION_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^(?:@\w+\s+)*(public\s+|internal\s+|fileprivate\s+|private\s+)?extension\s+(\w+)(?:\s*:\s*[^{]+)?\s*\{")
        .unwrap()
});

// actor ActorName {
static ACTOR_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^(?:@\w+\s+)*(public\s+|internal\s+|fileprivate\s+|private\s+)?actor\s+(\w+)(?:\s*:\s*[^{]+)?\s*\{")
        .unwrap()
});

// func functionName(params) -> ReturnType {
static FUNC_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^(?:\s*)(?:@\w+\s+)*(public\s+|internal\s+|fileprivate\s+|private\s+|open\s+)?(?:static\s+|class\s+)?(?:override\s+)?func\s+(\w+)\s*\([^)]*\)(?:\s*(?:async\s+)?(?:throws\s+)?(?:->\s*[^{]+)?)?\s*\{")
        .unwrap()
});

// init(params) {
static INIT_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^(?:\s*)(?:@\w+\s+)*(public\s+|internal\s+|fileprivate\s+|private\s+)?(?:convenience\s+|required\s+)?init\??\s*\([^)]*\)(?:\s*throws)?\s*\{")
        .unwrap()
});

impl LanguageParser for SwiftParser {
    fn parse_symbols(&self, content: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();

        for cap in CLASS_PATTERN.captures_iter(content) {
            let visibility = parse_visibility(cap.get(1).map(|m| m.as_str()));
            let name = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let line = line_number_at_offset(content, cap.get(0).unwrap().start());

            let end_line = find_brace_end(content, cap.get(0).unwrap().end() - 1)
                .map(|pos| line_number_at_offset(content, pos))
                .unwrap_or(line);

            let mut sym = Symbol::new(SymbolKind::Class, name.to_string(), line, visibility);
            sym = sym.with_line_range(line, end_line);
            sym = sym.with_signature(format!("class {}", name));
            symbols.push(sym);
        }

        for cap in STRUCT_PATTERN.captures_iter(content) {
            let visibility = parse_visibility(cap.get(1).map(|m| m.as_str()));
            let name = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let line = line_number_at_offset(content, cap.get(0).unwrap().start());

            let end_line = find_brace_end(content, cap.get(0).unwrap().end() - 1)
                .map(|pos| line_number_at_offset(content, pos))
                .unwrap_or(line);

            let mut sym = Symbol::new(SymbolKind::Struct, name.to_string(), line, visibility);
            sym = sym.with_line_range(line, end_line);
            sym = sym.with_signature(format!("struct {}", name));
            symbols.push(sym);
        }

        for cap in ENUM_PATTERN.captures_iter(content) {
            let visibility = parse_visibility(cap.get(1).map(|m| m.as_str()));
            let name = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let line = line_number_at_offset(content, cap.get(0).unwrap().start());

            let end_line = find_brace_end(content, cap.get(0).unwrap().end() - 1)
                .map(|pos| line_number_at_offset(content, pos))
                .unwrap_or(line);

            let mut sym = Symbol::new(SymbolKind::Enum, name.to_string(), line, visibility);
            sym = sym.with_line_range(line, end_line);
            sym = sym.with_signature(format!("enum {}", name));
            symbols.push(sym);
        }

        for cap in PROTOCOL_PATTERN.captures_iter(content) {
            let visibility = parse_visibility(cap.get(1).map(|m| m.as_str()));
            let name = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let line = line_number_at_offset(content, cap.get(0).unwrap().start());

            let end_line = find_brace_end(content, cap.get(0).unwrap().end() - 1)
                .map(|pos| line_number_at_offset(content, pos))
                .unwrap_or(line);

            let mut sym = Symbol::new(SymbolKind::Trait, name.to_string(), line, visibility);
            sym = sym.with_line_range(line, end_line);
            sym = sym.with_signature(format!("protocol {}", name));
            symbols.push(sym);
        }

        for cap in EXTENSION_PATTERN.captures_iter(content) {
            let visibility = parse_visibility(cap.get(1).map(|m| m.as_str()));
            let name = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let line = line_number_at_offset(content, cap.get(0).unwrap().start());

            let end_line = find_brace_end(content, cap.get(0).unwrap().end() - 1)
                .map(|pos| line_number_at_offset(content, pos))
                .unwrap_or(line);

            let mut sym = Symbol::new(
                SymbolKind::Module,
                format!("extension {}", name),
                line,
                visibility,
            );
            sym = sym.with_line_range(line, end_line);
            symbols.push(sym);
        }

        for cap in ACTOR_PATTERN.captures_iter(content) {
            let visibility = parse_visibility(cap.get(1).map(|m| m.as_str()));
            let name = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let line = line_number_at_offset(content, cap.get(0).unwrap().start());

            let end_line = find_brace_end(content, cap.get(0).unwrap().end() - 1)
                .map(|pos| line_number_at_offset(content, pos))
                .unwrap_or(line);

            let mut sym = Symbol::new(SymbolKind::Class, name.to_string(), line, visibility);
            sym = sym.with_line_range(line, end_line);
            sym = sym.with_signature(format!("actor {}", name));
            symbols.push(sym);
        }

        for cap in FUNC_PATTERN.captures_iter(content) {
            let visibility = parse_visibility(cap.get(1).map(|m| m.as_str()));
            let name = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let line = line_number_at_offset(content, cap.get(0).unwrap().start());

            let end_line = find_brace_end(content, cap.get(0).unwrap().end() - 1)
                .map(|pos| line_number_at_offset(content, pos))
                .unwrap_or(line);

            let full_match = cap.get(0).unwrap().as_str().trim();
            let signature = full_match.trim_end_matches('{').trim().to_string();

            let mut sym = Symbol::new(SymbolKind::Function, name.to_string(), line, visibility);
            sym = sym.with_line_range(line, end_line);
            sym = sym.with_signature(signature);
            symbols.push(sym);
        }

        for cap in INIT_PATTERN.captures_iter(content) {
            let visibility = parse_visibility(cap.get(1).map(|m| m.as_str()));
            let line = line_number_at_offset(content, cap.get(0).unwrap().start());

            let end_line = find_brace_end(content, cap.get(0).unwrap().end() - 1)
                .map(|pos| line_number_at_offset(content, pos))
                .unwrap_or(line);

            let full_match = cap.get(0).unwrap().as_str().trim();
            let signature = full_match.trim_end_matches('{').trim().to_string();

            let mut sym = Symbol::new(SymbolKind::Method, "init".to_string(), line, visibility);
            sym = sym.with_line_range(line, end_line);
            sym = sym.with_signature(signature);
            symbols.push(sym);
        }

        symbols.sort_by_key(|s| s.line_range.start);
        symbols.dedup_by(|a, b| a.name == b.name && a.line_range.start == b.line_range.start);
        symbols
    }
}

fn parse_visibility(modifier: Option<&str>) -> Visibility {
    match modifier.map(|s| s.trim()) {
        Some("public") | Some("open") => Visibility::Public,
        Some("private") => Visibility::Private,
        Some("fileprivate") => Visibility::Private,
        Some("internal") | None => Visibility::Internal,
        _ => Visibility::Internal,
    }
}

fn line_number_at_offset(content: &str, offset: usize) -> usize {
    content[..offset].matches('\n').count() + 1
}

fn find_brace_end(content: &str, start: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut depth = 0;
    let mut in_string = false;
    let mut string_char = b'"';
    let mut i = start;

    while i < bytes.len() {
        let b = bytes[i];

        if in_string {
            if b == string_char && (i == 0 || bytes[i - 1] != b'\\') {
                in_string = false;
            }
            i += 1;
            continue;
        }

        match b {
            b'"' => {
                in_string = true;
                string_char = b;
            }
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}
