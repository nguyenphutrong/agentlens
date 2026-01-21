use anyhow::Result;
use chrono::Utc;
use std::fs;
use std::path::Path;
use std::sync::Arc;

use crate::analyze::extract_symbols;
use crate::config::ChunkingConfig;
use crate::scan::scan_directory;
use crate::types::FileEntry;

use super::chunker::{ChunkInfo, Chunker};
use super::embedder::Embedder;
use super::store::{Chunk, Document, VectorStore};

pub struct Indexer {
    store: Arc<dyn VectorStore>,
    embedder: Arc<dyn Embedder>,
    chunker: Chunker,
}

pub struct IndexResult {
    pub files_processed: usize,
    pub chunks_created: usize,
    pub files_skipped: usize,
    pub errors: Vec<String>,
}

impl Indexer {
    pub fn new(
        store: Arc<dyn VectorStore>,
        embedder: Arc<dyn Embedder>,
        config: &ChunkingConfig,
    ) -> Self {
        let chunker = Chunker::from_tokens(config.max_tokens, config.overlap_tokens);
        Self {
            store,
            embedder,
            chunker,
        }
    }

    /// Index all files in a directory
    pub async fn index_all(
        &self,
        root: &Path,
        respect_gitignore: bool,
        force: bool,
    ) -> Result<IndexResult> {
        let files = scan_directory(root, 500, respect_gitignore, None)?;

        let mut result = IndexResult {
            files_processed: 0,
            chunks_created: 0,
            files_skipped: 0,
            errors: Vec::new(),
        };

        // Load existing index
        self.store.load().await?;

        for file in files {
            match self.index_file(&file, force).await {
                Ok(Some(chunks_count)) => {
                    result.files_processed += 1;
                    result.chunks_created += chunks_count;
                }
                Ok(None) => {
                    result.files_skipped += 1;
                }
                Err(e) => {
                    result.errors.push(format!("{}: {}", file.relative_path, e));
                }
            }
        }

        // Persist the index
        self.store.persist().await?;

        Ok(result)
    }

    /// Index a single file
    /// Returns Some(chunk_count) if indexed, None if skipped (unchanged)
    pub async fn index_file(&self, file: &FileEntry, force: bool) -> Result<Option<usize>> {
        let content = fs::read_to_string(&file.path)?;
        let content_hash = hash_content(&content);

        // Check if file has changed
        if !force {
            if let Some(doc) = self.store.get_document(&file.relative_path).await? {
                if doc.hash == content_hash {
                    return Ok(None); // File unchanged
                }
            }
        }

        // Delete old chunks for this file
        self.store.delete_by_file(&file.relative_path).await?;

        // Extract symbols for symbol-based chunking
        let symbols = extract_symbols(file, &content);

        // Create chunks
        let chunk_infos = self.chunker.chunk_by_symbols(file, &content, &symbols);

        if chunk_infos.is_empty() {
            return Ok(Some(0));
        }

        // Embed chunks in batches
        let chunks = self.embed_chunks(chunk_infos).await?;
        let chunk_count = chunks.len();
        let chunk_ids: Vec<String> = chunks.iter().map(|c| c.id.clone()).collect();

        // Save chunks
        self.store.save_chunks(chunks).await?;

        // Save document metadata
        let doc = Document {
            path: file.relative_path.clone(),
            hash: content_hash,
            mod_time: Utc::now(),
            chunk_ids,
        };
        self.store.save_document(doc).await?;

        Ok(Some(chunk_count))
    }

    /// Embed chunks and return full Chunk objects
    async fn embed_chunks(&self, chunk_infos: Vec<ChunkInfo>) -> Result<Vec<Chunk>> {
        const BATCH_SIZE: usize = 32;
        let mut chunks = Vec::with_capacity(chunk_infos.len());

        for batch in chunk_infos.chunks(BATCH_SIZE) {
            let texts: Vec<String> = batch.iter().map(|c| c.content.clone()).collect();
            let embeddings = self.embedder.embed_batch(&texts).await?;

            for (info, vector) in batch.iter().zip(embeddings.into_iter()) {
                chunks.push(Chunk {
                    id: info.id.clone(),
                    file_path: info.file_path.clone(),
                    start_line: info.start_line,
                    end_line: info.end_line,
                    content: info.content.clone(),
                    vector,
                    hash: info.hash.clone(),
                    updated_at: Utc::now(),
                    chunk_type: info.chunk_type.clone(),
                });
            }
        }

        Ok(chunks)
    }

    /// Remove files from index that no longer exist
    pub async fn prune_deleted(&self, root: &Path, respect_gitignore: bool) -> Result<usize> {
        let existing_files = scan_directory(root, 500, respect_gitignore, None)?;
        let existing_paths: std::collections::HashSet<String> = existing_files
            .iter()
            .map(|f| f.relative_path.clone())
            .collect();

        let indexed_paths = self.store.list_documents().await?;
        let mut pruned = 0;

        for path in indexed_paths {
            if !existing_paths.contains(&path) {
                self.store.delete_by_file(&path).await?;
                pruned += 1;
            }
        }

        if pruned > 0 {
            self.store.persist().await?;
        }

        Ok(pruned)
    }
}

fn hash_content(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())[..16].to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_content() {
        let h1 = hash_content("hello");
        let h2 = hash_content("hello");
        let h3 = hash_content("world");

        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
        assert_eq!(h1.len(), 16);
    }
}
