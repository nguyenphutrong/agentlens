use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ChunkType {
    Function,
    Class,
    Method,
    Module,
    FileHeader,
    Block,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    pub id: String,
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub content: String,
    pub vector: Vec<f32>,
    pub hash: String,
    pub updated_at: DateTime<Utc>,
    pub chunk_type: ChunkType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Document {
    pub path: String,
    pub hash: String,
    pub mod_time: DateTime<Utc>,
    pub chunk_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub chunk: Chunk,
    pub score: f32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IndexStats {
    pub total_files: usize,
    pub total_chunks: usize,
    pub index_size_bytes: u64,
    pub last_updated: Option<DateTime<Utc>>,
}

impl SearchResult {
    pub fn new(chunk: Chunk, score: f32) -> Self {
        Self { chunk, score }
    }
}
