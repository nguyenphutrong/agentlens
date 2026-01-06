use crate::analyze::lang::LanguageParser;
use crate::types::{Symbol, SymbolKind, Visibility};
use once_cell::sync::Lazy;
use regex::Regex;

pub struct DartParser;

// class ClassName { or abstract class ClassName {
static CLASS_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^(abstract\s+)?class\s+(\w+)(?:\s+extends\s+\w+)?(?:\s+(?:with|implements)\s+[^{]+)?\s*\{")
        .unwrap()
});

// mixin MixinName on BaseClass {
static MIXIN_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^mixin\s+(\w+)(?:\s+on\s+\w+)?\s*\{").unwrap());

// extension ExtensionName on Type {
static EXTENSION_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^extension\s+(\w+)\s+on\s+\w+[^{]*\{").unwrap());

// enum EnumName {
static ENUM_PATTERN: Lazy<Regex> = Lazy::new(|| Regex::new(r"(?m)^enum\s+(\w+)\s*\{").unwrap());

// ReturnType functionName(params) { or void functionName(params) async {
static FUNCTION_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^(static\s+)?(\w+(?:<[^>]+>)?)\s+(\w+)\s*\([^)]*\)(?:\s*async)?\s*\{").unwrap()
});

// factory ClassName.namedConstructor(params) { or ClassName(params) {
static CONSTRUCTOR_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?m)^\s*(factory\s+)?(\w+)(?:\.(\w+))?\s*\([^)]*\)(?:\s*:\s*[^{]+)?\s*\{").unwrap()
});

// get propertyName { or Type get propertyName {
static GETTER_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^\s*(?:\w+\s+)?get\s+(\w+)\s*\{").unwrap());

// set propertyName(value) {
static SETTER_PATTERN: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"(?m)^\s*set\s+(\w+)\s*\([^)]*\)\s*\{").unwrap());

impl LanguageParser for DartParser {
    fn parse_symbols(&self, content: &str) -> Vec<Symbol> {
        let mut symbols = Vec::new();

        for cap in CLASS_PATTERN.captures_iter(content) {
            let is_abstract = cap.get(1).is_some();
            let name = cap.get(2).map(|m| m.as_str()).unwrap_or("");
            let line = line_number_at_offset(content, cap.get(0).unwrap().start());
            let visibility = get_visibility(name);

            let end_line = find_brace_end(content, cap.get(0).unwrap().end() - 1)
                .map(|pos| line_number_at_offset(content, pos))
                .unwrap_or(line);

            let prefix = if is_abstract {
                "abstract class"
            } else {
                "class"
            };
            let mut sym = Symbol::new(SymbolKind::Class, name.to_string(), line, visibility);
            sym = sym.with_line_range(line, end_line);
            sym = sym.with_signature(format!("{} {}", prefix, name));
            symbols.push(sym);
        }

        for cap in MIXIN_PATTERN.captures_iter(content) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let line = line_number_at_offset(content, cap.get(0).unwrap().start());
            let visibility = get_visibility(name);

            let end_line = find_brace_end(content, cap.get(0).unwrap().end() - 1)
                .map(|pos| line_number_at_offset(content, pos))
                .unwrap_or(line);

            let mut sym = Symbol::new(SymbolKind::Trait, name.to_string(), line, visibility);
            sym = sym.with_line_range(line, end_line);
            sym = sym.with_signature(format!("mixin {}", name));
            symbols.push(sym);
        }

        for cap in EXTENSION_PATTERN.captures_iter(content) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let line = line_number_at_offset(content, cap.get(0).unwrap().start());

            let end_line = find_brace_end(content, cap.get(0).unwrap().end() - 1)
                .map(|pos| line_number_at_offset(content, pos))
                .unwrap_or(line);

            let mut sym = Symbol::new(
                SymbolKind::Module,
                name.to_string(),
                line,
                Visibility::Public,
            );
            sym = sym.with_line_range(line, end_line);
            sym = sym.with_signature(format!("extension {}", name));
            symbols.push(sym);
        }

        for cap in ENUM_PATTERN.captures_iter(content) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let line = line_number_at_offset(content, cap.get(0).unwrap().start());
            let visibility = get_visibility(name);

            let end_line = find_brace_end(content, cap.get(0).unwrap().end() - 1)
                .map(|pos| line_number_at_offset(content, pos))
                .unwrap_or(line);

            let mut sym = Symbol::new(SymbolKind::Enum, name.to_string(), line, visibility);
            sym = sym.with_line_range(line, end_line);
            sym = sym.with_signature(format!("enum {}", name));
            symbols.push(sym);
        }

        for cap in FUNCTION_PATTERN.captures_iter(content) {
            let is_static = cap.get(1).is_some();
            let name = cap.get(3).map(|m| m.as_str()).unwrap_or("");
            let line = line_number_at_offset(content, cap.get(0).unwrap().start());

            if is_control_flow_keyword(name) {
                continue;
            }

            let visibility = get_visibility(name);
            let kind = if is_static {
                SymbolKind::Function
            } else {
                SymbolKind::Method
            };

            let end_line = find_brace_end(content, cap.get(0).unwrap().end() - 1)
                .map(|pos| line_number_at_offset(content, pos))
                .unwrap_or(line);

            let full_match = cap.get(0).unwrap().as_str().trim();
            let signature = full_match.trim_end_matches('{').trim().to_string();

            let mut sym = Symbol::new(kind, name.to_string(), line, visibility);
            sym = sym.with_line_range(line, end_line);
            sym = sym.with_signature(signature);
            symbols.push(sym);
        }

        for cap in GETTER_PATTERN.captures_iter(content) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let line = line_number_at_offset(content, cap.get(0).unwrap().start());
            let visibility = get_visibility(name);

            let end_line = find_brace_end(content, cap.get(0).unwrap().end() - 1)
                .map(|pos| line_number_at_offset(content, pos))
                .unwrap_or(line);

            let mut sym = Symbol::new(
                SymbolKind::Method,
                format!("get {}", name),
                line,
                visibility,
            );
            sym = sym.with_line_range(line, end_line);
            symbols.push(sym);
        }

        for cap in SETTER_PATTERN.captures_iter(content) {
            let name = cap.get(1).map(|m| m.as_str()).unwrap_or("");
            let line = line_number_at_offset(content, cap.get(0).unwrap().start());
            let visibility = get_visibility(name);

            let end_line = find_brace_end(content, cap.get(0).unwrap().end() - 1)
                .map(|pos| line_number_at_offset(content, pos))
                .unwrap_or(line);

            let mut sym = Symbol::new(
                SymbolKind::Method,
                format!("set {}", name),
                line,
                visibility,
            );
            sym = sym.with_line_range(line, end_line);
            symbols.push(sym);
        }

        symbols.sort_by_key(|s| s.line_range.start);
        symbols.dedup_by(|a, b| a.name == b.name && a.line_range.start == b.line_range.start);
        symbols
    }
}

fn get_visibility(name: &str) -> Visibility {
    if name.starts_with('_') {
        Visibility::Private
    } else {
        Visibility::Public
    }
}

fn is_control_flow_keyword(name: &str) -> bool {
    matches!(name, "if" | "for" | "while" | "switch" | "catch" | "try")
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
