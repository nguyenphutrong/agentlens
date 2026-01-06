use crate::analyze::lang::LanguageParser;
use crate::types::{Symbol, SymbolKind, Visibility};
use once_cell::sync::Lazy;
use regex::Regex;

pub struct JavaParser;

static CLASS_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(
        r"(?m)^\s*(public|private|protected)?\s*(abstract|final)?\s*(class|interface|enum)\s+(\w+)",
    )
    .unwrap()
});

static METHOD_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^\s*(public|private|protected)?\s*(static)?\s*(final)?\s*(abstract)?\s*(\w+(?:<[^>]+>)?)\s+(\w+)\s*\(")
        .unwrap()
});

static ANNOTATION_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^\s*@interface\s+(\w+)").unwrap());

impl LanguageParser for JavaParser {
    fn parse_symbols(&self, content: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();

        for cap in CLASS_PATTERN.captures_iter(content) {
            let visibility_str = cap.get(1).map(|m| m.as_str()).unwrap_or("package");
            let kind_str = cap.get(3).map(|m| m.as_str()).unwrap_or("class");
            let name = cap.get(4).map(|m| m.as_str()).unwrap_or("");
            let line = line_number_at_offset(content, cap.get(0).unwrap().start());

            let visibility = match visibility_str {
                "private" => Visibility::Private,
                "protected" => Visibility::Protected,
                "public" => Visibility::Public,
                _ => Visibility::Internal,
            };

            let kind = match kind_str {
                "interface" => SymbolKind::Interface,
                "enum" => SymbolKind::Enum,
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
            let visibility_str = cap.get(1).map(|m| m.as_str()).unwrap_or("package");
            let return_type = cap.get(5).map(|m| m.as_str()).unwrap_or("");
            let name = cap.get(6).map(|m| m.as_str()).unwrap_or("");
            let line = line_number_at_offset(content, cap.get(0).unwrap().start());

            if name == "if"
                || name == "for"
                || name == "while"
                || name == "switch"
                || name == "catch"
            {
                continue;
            }

            if return_type == "class"
                || return_type == "interface"
                || return_type == "enum"
                || return_type == "new"
            {
                continue;
            }

            let visibility = match visibility_str {
                "private" => Visibility::Private,
                "protected" => Visibility::Protected,
                "public" => Visibility::Public,
                _ => Visibility::Internal,
            };

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

        for cap in ANNOTATION_PATTERN.captures_iter(content) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let line = line_number_at_offset(content, cap.get(0).unwrap().start());

            let sym = Symbol::new(
                SymbolKind::Interface,
                name.to_string(),
                line,
                Visibility::Public,
            );
            symbols.push(sym);
        }

        symbols.sort_by_key(|s| s.line_range.start);
        symbols
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
