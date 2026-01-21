use anyhow::{anyhow, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::Embedder;

pub struct OllamaEmbedder {
    endpoint: String,
    model: String,
    dimensions: usize,
    client: Client,
}

#[derive(Serialize)]
struct EmbedRequest {
    model: String,
    input: Vec<String>,
    truncate: bool,
}

#[derive(Deserialize)]
struct EmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

#[derive(Deserialize)]
struct OllamaTagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Deserialize)]
struct OllamaModel {
    name: String,
}

impl OllamaEmbedder {
    pub fn new(endpoint: &str, model: &str, dimensions: usize) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("Failed to create HTTP client");

        Self {
            endpoint: endpoint.trim_end_matches('/').to_string(),
            model: model.to_string(),
            dimensions,
            client,
        }
    }
}

#[async_trait]
impl Embedder for OllamaEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let results = self.embed_batch(&[text.to_string()]).await?;
        results
            .into_iter()
            .next()
            .ok_or_else(|| anyhow!("No embedding returned"))
    }

    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let request = EmbedRequest {
            model: self.model.clone(),
            input: texts.to_vec(),
            truncate: true,
        };

        let response = self
            .client
            .post(format!("{}/api/embed", self.endpoint))
            .json(&request)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() {
                    anyhow!(
                        "Cannot connect to Ollama at {}. Is Ollama running?\n\
                         Install: https://ollama.ai\n\
                         Start: ollama serve",
                        self.endpoint
                    )
                } else {
                    anyhow!("Ollama request failed: {}", e)
                }
            })?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();

            if status.as_u16() == 404 || body.contains("not found") {
                return Err(anyhow!(
                    "Model '{}' not found. Pull it with:\n  ollama pull {}",
                    self.model,
                    self.model
                ));
            }

            return Err(anyhow!("Ollama error ({}): {}", status, body));
        }

        let embed_response: EmbedResponse = response.json().await?;
        Ok(embed_response.embeddings)
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    async fn health_check(&self) -> Result<()> {
        let response = self
            .client
            .get(format!("{}/api/tags", self.endpoint))
            .send()
            .await
            .map_err(|_| {
                anyhow!(
                    "Cannot connect to Ollama at {}. Is Ollama running?\n\
                     Install: https://ollama.ai\n\
                     Start: ollama serve",
                    self.endpoint
                )
            })?;

        if !response.status().is_success() {
            return Err(anyhow!("Ollama health check failed"));
        }

        let tags: OllamaTagsResponse = response.json().await?;
        let model_available = tags
            .models
            .iter()
            .any(|m| m.name.starts_with(&self.model) || m.name == format!("{}:latest", self.model));

        if !model_available {
            return Err(anyhow!(
                "Model '{}' not installed. Pull it with:\n  ollama pull {}",
                self.model,
                self.model
            ));
        }

        Ok(())
    }
}
