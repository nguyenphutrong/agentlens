use crate::analyze::lang::LanguageParser;
use crate::types::{Symbol, SymbolKind, Visibility};
use once_cell::sync::Lazy;
use regex::Regex;

pub struct CSharpParser;

static NAMESPACE_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^\s*namespace\s+([\w.]+)").unwrap());

static CLASS_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^\s*(public|private|protected|internal)?\s*(abstract|sealed|static|partial)?\s*(class|interface|enum|struct|record)\s+(\w+)")
        .unwrap()
});

static METHOD_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^\s*(public|private|protected|internal)?\s*(static|virtual|override|abstract|async)?\s*([\w<>\[\],\s]+?)\s+(\w+)\s*\(")
        .unwrap()
});

static PROPERTY_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^\s*(public|private|protected|internal)?\s*(static|virtual|override)?\s*([\w<>\[\]?]+)\s+(\w+)\s*\{\s*(get|set)")
        .unwrap()
});

impl LanguageParser for CSharpParser {
    fn parse_symbols(&self, content: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();

        for cap in NAMESPACE_PATTERN.captures_iter(content) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let line = line_number_at_offset(content, cap.get(0).unwrap().start());

            let end_line = find_brace_end(content, cap.get(0).unwrap().end())
                .map(|pos| line_number_at_offset(content, pos))
                .unwrap_or(line);

            let mut sym = Symbol::new(
                SymbolKind::Module,
                name.to_string(),
                line,
                Visibility::Public,
            );
            sym = sym.with_line_range(line, end_line);
            sym = sym.with_signature(format!("namespace {}", name));
            symbols.push(sym);
        }

        for cap in CLASS_PATTERN.captures_iter(content) {
            let visibility_str = cap.get(1).map(|m| m.as_str()).unwrap_or("internal");
            let kind_str = cap.get(3).map(|m| m.as_str()).unwrap_or("class");
            let name = cap.get(4).map(|m| m.as_str()).unwrap_or("");
            let line = line_number_at_offset(content, cap.get(0).unwrap().start());

            let visibility = parse_visibility(visibility_str);

            let kind = match kind_str {
                "interface" => SymbolKind::Interface,
                "enum" => SymbolKind::Enum,
                "struct" | "record" => SymbolKind::Struct,
                _ => SymbolKind::Class,
            };

            let end_line = find_brace_end(content, cap.get(0).unwrap().end())
                .map(|pos| line_number_at_offset(content, pos))
                .unwrap_or(line);

            let full_match = cap.get(0).unwrap().as_str().trim();
            let mut sym = Symbol::new(kind, name.to_string(), line, visibility);
            sym = sym.with_line_range(line, end_line);
            sym = sym.with_signature(full_match.to_string());
            symbols.push(sym);
        }

        for cap in METHOD_PATTERN.captures_iter(content) {
            let visibility_str = cap.get(1).map(|m| m.as_str()).unwrap_or("private");
            let return_type = cap.get(3).map(|m| m.as_str()).unwrap_or("").trim();
            let name = cap.get(4).map(|m| m.as_str()).unwrap_or("");
            let line = line_number_at_offset(content, cap.get(0).unwrap().start());

            if name == "if"
                || name == "for"
                || name == "while"
                || name == "switch"
                || name == "catch"
                || name == "foreach"
            {
                continue;
            }

            if return_type == "class"
                || return_type == "interface"
                || return_type == "enum"
                || return_type == "struct"
                || return_type == "new"
                || return_type == "namespace"
            {
                continue;
            }

            let visibility = parse_visibility(visibility_str);

            let end_line = find_brace_end(content, cap.get(0).unwrap().end())
                .map(|pos| line_number_at_offset(content, pos))
                .unwrap_or(line);

            let full_match = cap.get(0).unwrap().as_str().trim();
            let signature = full_match.trim_end_matches('(').to_string() + "(...)";

            let mut sym = Symbol::new(SymbolKind::Method, name.to_string(), line, visibility);
            sym = sym.with_line_range(line, end_line);
            sym = sym.with_signature(signature);
            symbols.push(sym);
        }

        for cap in PROPERTY_PATTERN.captures_iter(content) {
            let visibility_str = cap.get(1).map(|m| m.as_str()).unwrap_or("private");
            let name = cap.get(4).map(|m| m.as_str()).unwrap_or("");
            let line = line_number_at_offset(content, cap.get(0).unwrap().start());

            let visibility = parse_visibility(visibility_str);

            let sym = Symbol::new(SymbolKind::Const, name.to_string(), line, visibility);
            symbols.push(sym);
        }

        symbols.sort_by_key(|s| s.line_range.start);
        symbols
    }
}

fn parse_visibility(s: &str) -> Visibility {
    match s {
        "private" => Visibility::Private,
        "protected" => Visibility::Protected,
        "public" => Visibility::Public,
        "internal" => Visibility::Internal,
        _ => Visibility::Private,
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
            b'"' | b'\'' => {
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
