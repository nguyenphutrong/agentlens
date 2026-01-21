use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::RwLock;

use super::{cosine_similarity, Chunk, Document, IndexStats, SearchResult, VectorStore};

#[derive(Debug, Default, Serialize, Deserialize)]
struct IndexData {
    chunks: HashMap<String, Chunk>,
    documents: HashMap<String, Document>,
}

pub struct GobStore {
    path: PathBuf,
    data: RwLock<IndexData>,
}

impl GobStore {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            data: RwLock::new(IndexData::default()),
        }
    }

    fn atomic_write(&self, data: &IndexData) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        let temp_path = self.path.with_extension("tmp");
        let json = serde_json::to_vec(data)?;
        fs::write(&temp_path, json)?;
        fs::rename(temp_path, &self.path)?;

        Ok(())
    }
}

#[async_trait]
impl VectorStore for GobStore {
    async fn save_chunks(&self, chunks: Vec<Chunk>) -> Result<()> {
        let mut data = self.data.write().map_err(|e| anyhow::anyhow!("{}", e))?;
        for chunk in chunks {
            data.chunks.insert(chunk.id.clone(), chunk);
        }
        Ok(())
    }

    async fn delete_by_file(&self, file_path: &str) -> Result<()> {
        let mut data = self.data.write().map_err(|e| anyhow::anyhow!("{}", e))?;

        let chunk_ids_to_remove: Vec<String> = data
            .chunks
            .iter()
            .filter(|(_, c)| c.file_path == file_path)
            .map(|(id, _)| id.clone())
            .collect();

        for id in chunk_ids_to_remove {
            data.chunks.remove(&id);
        }

        data.documents.remove(file_path);

        Ok(())
    }

    async fn search(&self, query_vector: &[f32], limit: usize) -> Result<Vec<SearchResult>> {
        let data = self.data.read().map_err(|e| anyhow::anyhow!("{}", e))?;

        let mut results: Vec<SearchResult> = data
            .chunks
            .values()
            .map(|chunk| {
                let score = cosine_similarity(query_vector, &chunk.vector);
                SearchResult::new(chunk.clone(), score)
            })
            .collect();

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);

        Ok(results)
    }

    async fn get_document(&self, file_path: &str) -> Result<Option<Document>> {
        let data = self.data.read().map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(data.documents.get(file_path).cloned())
    }

    async fn save_document(&self, doc: Document) -> Result<()> {
        let mut data = self.data.write().map_err(|e| anyhow::anyhow!("{}", e))?;
        data.documents.insert(doc.path.clone(), doc);
        Ok(())
    }

    async fn list_documents(&self) -> Result<Vec<String>> {
        let data = self.data.read().map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(data.documents.keys().cloned().collect())
    }

    async fn get_all_chunks(&self) -> Result<Vec<Chunk>> {
        let data = self.data.read().map_err(|e| anyhow::anyhow!("{}", e))?;
        Ok(data.chunks.values().cloned().collect())
    }

    async fn persist(&self) -> Result<()> {
        let data = self.data.read().map_err(|e| anyhow::anyhow!("{}", e))?;
        self.atomic_write(&data)
    }

    async fn load(&self) -> Result<()> {
        if !self.path.exists() {
            return Ok(());
        }

        let content = fs::read(&self.path)?;
        let loaded: IndexData = serde_json::from_slice(&content)?;

        let mut data = self.data.write().map_err(|e| anyhow::anyhow!("{}", e))?;
        *data = loaded;

        Ok(())
    }

    async fn stats(&self) -> Result<IndexStats> {
        let data = self.data.read().map_err(|e| anyhow::anyhow!("{}", e))?;

        let index_size = if self.path.exists() {
            fs::metadata(&self.path)?.len()
        } else {
            0
        };

        let last_updated = data
            .chunks
            .values()
            .map(|c| c.updated_at)
            .max();

        Ok(IndexStats {
            total_files: data.documents.len(),
            total_chunks: data.chunks.len(),
            index_size_bytes: index_size,
            last_updated,
        })
    }

    async fn clear(&self) -> Result<()> {
        let mut data = self.data.write().map_err(|e| anyhow::anyhow!("{}", e))?;
        data.chunks.clear();
        data.documents.clear();

        if self.path.exists() {
            fs::remove_file(&self.path)?;
        }

        Ok(())
    }
}
