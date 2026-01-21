use crate::types::{FileEntry, Symbol, SymbolKind};
use sha2::{Digest, Sha256};

use super::store::ChunkType;

/// Raw chunk info before embedding
#[derive(Debug, Clone)]
pub struct ChunkInfo {
    pub id: String,
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub content: String,
    pub hash: String,
    pub chunk_type: ChunkType,
}

pub struct Chunker {
    max_chars: usize,
    overlap_chars: usize,
}

impl Default for Chunker {
    fn default() -> Self {
        Self::new(2048, 200)
    }
}

impl Chunker {
    pub fn new(max_chars: usize, overlap_chars: usize) -> Self {
        Self {
            max_chars,
            overlap_chars,
        }
    }

    /// Create chunker from token config (rough conversion: 1 token ~ 4 chars)
    pub fn from_tokens(max_tokens: usize, overlap_tokens: usize) -> Self {
        Self::new(max_tokens * 4, overlap_tokens * 4)
    }

    /// Chunk by symbols (functions, classes) - preferred for code
    pub fn chunk_by_symbols(
        &self,
        file: &FileEntry,
        content: &str,
        symbols: &[Symbol],
    ) -> Vec<ChunkInfo> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        // Process function-level symbols
        let functions: Vec<&Symbol> = symbols
            .iter()
            .filter(|s| {
                matches!(
                    s.kind,
                    SymbolKind::Function
                        | SymbolKind::Method
                        | SymbolKind::Class
                        | SymbolKind::Struct
                )
            })
            .collect();

        for symbol in functions {
            let start_idx = symbol.line_range.start.saturating_sub(1);
            let end_idx = symbol.line_range.end.min(lines.len());

            if start_idx >= lines.len() || start_idx >= end_idx {
                continue;
            }

            let chunk_lines: Vec<&str> = lines[start_idx..end_idx].to_vec();
            let chunk_content = chunk_lines.join("\n");

            if chunk_content.trim().is_empty() {
                continue;
            }

            // If chunk is too large, split it
            if chunk_content.len() > self.max_chars {
                let sub_chunks = self.split_large_chunk(
                    &file.relative_path,
                    &chunk_content,
                    start_idx + 1,
                    symbol_to_chunk_type(symbol.kind),
                );
                chunks.extend(sub_chunks);
            } else {
                let formatted = format!(
                    "File: {}\nSymbol: {} ({})\nLines: {}-{}\n\n{}",
                    file.relative_path,
                    symbol.name,
                    symbol.kind,
                    start_idx + 1,
                    end_idx,
                    chunk_content
                );

                chunks.push(ChunkInfo {
                    id: format!(
                        "{}:{}:{}",
                        file.relative_path, symbol.name, symbol.line_range.start
                    ),
                    file_path: file.relative_path.clone(),
                    start_line: start_idx + 1,
                    end_line: end_idx,
                    content: formatted,
                    hash: hash_content(&chunk_content),
                    chunk_type: symbol_to_chunk_type(symbol.kind),
                });
            }
        }

        // If no symbol chunks, fall back to window-based chunking
        if chunks.is_empty() {
            chunks = self.chunk_by_window(file, content);
        }

        chunks
    }

    /// Fallback: sliding window chunking for files without symbols
    pub fn chunk_by_window(&self, file: &FileEntry, content: &str) -> Vec<ChunkInfo> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();

        if lines.is_empty() {
            return chunks;
        }

        let mut start_line = 0;

        while start_line < lines.len() {
            let mut current_len = 0;
            let mut end_line = start_line;

            // Accumulate lines until max_chars
            while end_line < lines.len() && current_len < self.max_chars {
                current_len += lines[end_line].len() + 1; // +1 for newline
                end_line += 1;
            }

            let chunk_lines: Vec<&str> = lines[start_line..end_line].to_vec();
            let chunk_content = chunk_lines.join("\n");

            if !chunk_content.trim().is_empty() {
                let formatted = format!(
                    "File: {}\nLines: {}-{}\n\n{}",
                    file.relative_path,
                    start_line + 1,
                    end_line,
                    chunk_content
                );

                chunks.push(ChunkInfo {
                    id: format!("{}:block:{}", file.relative_path, start_line + 1),
                    file_path: file.relative_path.clone(),
                    start_line: start_line + 1,
                    end_line,
                    content: formatted,
                    hash: hash_content(&chunk_content),
                    chunk_type: ChunkType::Block,
                });
            }

            // Move forward with overlap
            let overlap_lines = self.overlap_chars / 80; // Assume ~80 chars per line
            start_line = end_line.saturating_sub(overlap_lines);

            if start_line >= lines.len() || end_line >= lines.len() {
                break;
            }
        }

        chunks
    }

    /// Split a large chunk into smaller pieces
    fn split_large_chunk(
        &self,
        file_path: &str,
        content: &str,
        base_line: usize,
        chunk_type: ChunkType,
    ) -> Vec<ChunkInfo> {
        let mut chunks = Vec::new();
        let lines: Vec<&str> = content.lines().collect();
        let mut start = 0;

        while start < lines.len() {
            let mut current_len = 0;
            let mut end = start;

            while end < lines.len() && current_len < self.max_chars {
                current_len += lines[end].len() + 1;
                end += 1;
            }

            let chunk_lines = &lines[start..end];
            let chunk_content = chunk_lines.join("\n");

            if !chunk_content.trim().is_empty() {
                let start_line = base_line + start;
                let end_line = base_line + end - 1;

                let formatted = format!(
                    "File: {}\nLines: {}-{}\n\n{}",
                    file_path, start_line, end_line, chunk_content
                );

                chunks.push(ChunkInfo {
                    id: format!("{}:split:{}", file_path, start_line),
                    file_path: file_path.to_string(),
                    start_line,
                    end_line,
                    content: formatted,
                    hash: hash_content(&chunk_content),
                    chunk_type: chunk_type.clone(),
                });
            }

            let overlap_lines = self.overlap_chars / 80;
            start = end.saturating_sub(overlap_lines);

            if start >= lines.len() {
                break;
            }
        }

        chunks
    }
}

fn symbol_to_chunk_type(kind: SymbolKind) -> ChunkType {
    match kind {
        SymbolKind::Function => ChunkType::Function,
        SymbolKind::Method => ChunkType::Method,
        SymbolKind::Class | SymbolKind::Struct => ChunkType::Class,
        SymbolKind::Module => ChunkType::Module,
        _ => ChunkType::Block,
    }
}

fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())[..16].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Visibility;
    use std::path::PathBuf;

    fn make_file(path: &str, lines: usize) -> FileEntry {
        FileEntry::new(PathBuf::from(path), path.to_string(), 1000, lines, 500)
    }

    #[test]
    fn test_chunk_by_window() {
        let chunker = Chunker::new(100, 20);
        let file = make_file("test.rs", 10);
        let content = "line 1\nline 2\nline 3\nline 4\nline 5\n\
                       line 6\nline 7\nline 8\nline 9\nline 10";

        let chunks = chunker.chunk_by_window(&file, content);
        assert!(!chunks.is_empty());
        assert!(chunks[0].content.contains("File: test.rs"));
    }

    #[test]
    fn test_chunk_by_symbols() {
        let chunker = Chunker::new(500, 50);
        let file = make_file("test.rs", 20);
        let content = "// header\n\
                       fn foo() {\n\
                           println!(\"hello\");\n\
                       }\n\
                       \n\
                       fn bar() {\n\
                           println!(\"world\");\n\
                       }";

        let symbols = vec![
            Symbol::new(
                SymbolKind::Function,
                "foo".to_string(),
                2,
                Visibility::Public,
            )
            .with_line_range(2, 4),
            Symbol::new(
                SymbolKind::Function,
                "bar".to_string(),
                6,
                Visibility::Public,
            )
            .with_line_range(6, 8),
        ];

        let chunks = chunker.chunk_by_symbols(&file, content, &symbols);
        assert_eq!(chunks.len(), 2);
        assert!(chunks[0].content.contains("foo"));
        assert!(chunks[1].content.contains("bar"));
    }

    #[test]
    fn test_hash_content() {
        let hash1 = hash_content("hello");
        let hash2 = hash_content("hello");
        let hash3 = hash_content("world");

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
        assert_eq!(hash1.len(), 16);
    }
}
