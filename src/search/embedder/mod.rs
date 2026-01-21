mod ollama;

pub use ollama::OllamaEmbedder;

use anyhow::Result;
use async_trait::async_trait;

#[async_trait]
pub trait Embedder: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>>;
    fn dimensions(&self) -> usize;
    async fn health_check(&self) -> Result<()>;
}

#[derive(Debug, Clone)]
pub struct EmbedderConfig {
    pub provider: String,
    pub model: String,
    pub endpoint: Option<String>,
    pub dimensions: usize,
}

impl Default for EmbedderConfig {
    fn default() -> Self {
        Self {
            provider: "ollama".to_string(),
            model: "nomic-embed-text".to_string(),
            endpoint: None,
            dimensions: 768,
        }
    }
}

pub fn create_embedder(config: &EmbedderConfig) -> Box<dyn Embedder> {
    let endpoint = config
        .endpoint
        .clone()
        .unwrap_or_else(|| "http://localhost:11434".to_string());
    Box::new(OllamaEmbedder::new(
        &endpoint,
        &config.model,
        config.dimensions,
    ))
}
