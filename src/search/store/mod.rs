mod gob;
mod types;

pub use gob::GobStore;
pub use types::{Chunk, ChunkType, Document, IndexStats, SearchResult};

use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn save_chunks(&self, chunks: Vec<Chunk>) -> Result<()>;
    async fn delete_by_file(&self, file_path: &str) -> Result<()>;
    async fn search(&self, query_vector: &[f32], limit: usize) -> Result<Vec<SearchResult>>;
    async fn get_document(&self, file_path: &str) -> Result<Option<Document>>;
    async fn save_document(&self, doc: Document) -> Result<()>;
    async fn list_documents(&self) -> Result<Vec<String>>;
    async fn get_all_chunks(&self) -> Result<Vec<Chunk>>;
    async fn persist(&self) -> Result<()>;
    async fn load(&self) -> Result<()>;
    async fn stats(&self) -> Result<IndexStats>;
    async fn clear(&self) -> Result<()>;
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }

    let mut dot = 0.0f32;
    let mut norm_a = 0.0f32;
    let mut norm_b = 0.0f32;

    for (x, y) in a.iter().zip(b.iter()) {
        dot += x * y;
        norm_a += x * x;
        norm_b += y * y;
    }

    let denom = norm_a.sqrt() * norm_b.sqrt();
    if denom == 0.0 {
        0.0
    } else {
        dot / denom
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity_identical() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_orthogonal() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_opposite() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![-1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - (-1.0)).abs() < 0.001);
    }

    #[test]
    fn test_cosine_similarity_empty() {
        let a: Vec<f32> = vec![];
        let b: Vec<f32> = vec![];
        assert_eq!(cosine_similarity(&a, &b), 0.0);
    }
}
