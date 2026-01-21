use anyhow::Result;
use std::sync::Arc;

use super::embedder::Embedder;
use super::hybrid::{reciprocal_rank_fusion, text_search};
use super::store::{SearchResult, VectorStore};

pub struct Searcher {
    store: Arc<dyn VectorStore>,
    embedder: Arc<dyn Embedder>,
    hybrid_enabled: bool,
    hybrid_k: f32,
}

impl Searcher {
    pub fn new(
        store: Arc<dyn VectorStore>,
        embedder: Arc<dyn Embedder>,
        hybrid_enabled: bool,
        hybrid_k: f32,
    ) -> Self {
        Self {
            store,
            embedder,
            hybrid_enabled,
            hybrid_k,
        }
    }

    /// Search with vector similarity only
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        // Load index if needed
        self.store.load().await?;

        // Embed the query
        let query_vector = self.embedder.embed(query).await?;

        // Vector search
        self.store.search(&query_vector, limit).await
    }

    /// Hybrid search: combines vector search with text search using RRF
    pub async fn search_hybrid(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        // Load index if needed
        self.store.load().await?;

        // Embed the query
        let query_vector = self.embedder.embed(query).await?;

        // Vector search (get more results for fusion)
        let vector_results = self.store.search(&query_vector, limit * 2).await?;

        if !self.hybrid_enabled {
            // Just return vector results, truncated
            let mut results = vector_results;
            results.truncate(limit);
            return Ok(results);
        }

        // Text search
        let all_chunks = self.store.get_all_chunks().await?;
        let text_results = text_search(&all_chunks, query, limit * 2);

        // Combine with RRF
        let combined = reciprocal_rank_fusion(self.hybrid_k, limit, vec![vector_results, text_results]);

        Ok(combined)
    }

    /// Smart search: uses hybrid if enabled, otherwise vector-only
    pub async fn smart_search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        if self.hybrid_enabled {
            self.search_hybrid(query, limit).await
        } else {
            self.search(query, limit).await
        }
    }
}

#[cfg(test)]
mod tests {
    // Integration tests would require mock store and embedder
    // Unit tests for Searcher logic are minimal since it orchestrates other components
}
