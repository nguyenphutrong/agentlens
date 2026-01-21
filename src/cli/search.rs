use anyhow::Result;
use console::{style, Emoji};
use std::path::Path;
use std::sync::Arc;

use crate::config::SearchOptionsConfig;
use crate::search::{create_embedder, EmbedderConfig, GobStore, Searcher};

static SEARCH: Emoji<'_, '_> = Emoji("🔍 ", "");
static FILE: Emoji<'_, '_> = Emoji("📄 ", "");

pub async fn run_search(
    path: &Path,
    query: &str,
    limit: usize,
    hybrid: bool,
    json: bool,
    output_dir: &str,
) -> Result<()> {
    // Setup store path
    let store_path = path.join(output_dir).join("index.json");

    if !store_path.exists() {
        anyhow::bail!("No search index found. Run `agentlens index` first to build the index.");
    }

    // Create embedder
    let embedder_config = EmbedderConfig::default();
    let embedder: Arc<dyn crate::search::Embedder> = Arc::from(create_embedder(&embedder_config));

    // Create store
    let store: Arc<dyn crate::search::VectorStore> = Arc::new(GobStore::new(store_path));

    // Create searcher
    let search_config = SearchOptionsConfig::default();
    let searcher = Searcher::new(
        store,
        embedder,
        if hybrid {
            true
        } else {
            search_config.hybrid_enabled
        },
        search_config.hybrid_k,
    );

    // Perform search
    let results = searcher.smart_search(query, limit).await?;

    if json {
        let output = serde_json::to_string_pretty(&results)?;
        println!("{}", output);
    } else {
        if results.is_empty() {
            println!("No results found for: {}", style(query).italic());
            return Ok(());
        }

        println!(
            "\n{}Found {} results for: {}\n",
            SEARCH,
            style(results.len()).cyan(),
            style(query).yellow().bold()
        );

        for (i, result) in results.iter().enumerate() {
            let chunk = &result.chunk;
            println!(
                "{} {}. {} {}",
                FILE,
                style(i + 1).dim(),
                style(&chunk.file_path).green(),
                style(format!("(L{}-{})", chunk.start_line, chunk.end_line)).dim()
            );
            println!(
                "   Score: {} | Type: {:?}",
                style(format!("{:.3}", result.score)).cyan(),
                chunk.chunk_type
            );

            // Show preview (first 200 chars of content, skip header)
            let preview_lines: Vec<&str> = chunk.content.lines().skip(3).take(5).collect();
            let preview = preview_lines.join("\n");
            if !preview.is_empty() {
                let truncated = if preview.len() > 200 {
                    format!("{}...", &preview[..200])
                } else {
                    preview
                };
                println!("   {}", style(truncated).dim());
            }
            println!();
        }
    }

    Ok(())
}
