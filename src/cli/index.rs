use anyhow::Result;
use console::{style, Emoji};
use indicatif::{ProgressBar, ProgressStyle};
use std::path::Path;
use std::sync::Arc;

use crate::config::ChunkingConfig;
use crate::search::{create_embedder, Embedder, EmbedderConfig, GobStore, Indexer, VectorStore};

static INDEXING: Emoji<'_, '_> = Emoji("📊 ", "");
static SUCCESS: Emoji<'_, '_> = Emoji("✅ ", "");
static ERROR: Emoji<'_, '_> = Emoji("❌ ", "");
static INFO: Emoji<'_, '_> = Emoji("ℹ️  ", "");

pub async fn run_index(
    path: &Path,
    force: bool,
    prune: bool,
    output_dir: &str,
    verbose: bool,
) -> Result<()> {
    let store_path = path.join(output_dir).join("index.json");

    // Create embedder and store
    let embedder_config = EmbedderConfig::default();
    let embedder: Arc<dyn Embedder> = Arc::from(create_embedder(&embedder_config));

    // Health check
    if verbose {
        println!("{}Checking Ollama connection...", INFO);
    }

    embedder.health_check().await?;

    let store: Arc<dyn VectorStore> = Arc::new(GobStore::new(store_path));

    // Create indexer
    let chunking_config = ChunkingConfig::default();
    let indexer = Indexer::new(Arc::clone(&store), Arc::clone(&embedder), &chunking_config);

    // Show progress
    let pb = ProgressBar::new_spinner();
    pb.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner:.green} {msg}")
            .unwrap(),
    );
    pb.set_message(format!("{}Indexing {}...", INDEXING, path.display()));
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    // Run indexing
    let result = indexer.index_all(path, true, force).await?;

    pb.finish_and_clear();

    // Report results
    println!("\n{}Indexing complete!\n", SUCCESS);
    println!(
        "  Files processed: {}",
        style(result.files_processed).green()
    );
    println!("  Chunks created:  {}", style(result.chunks_created).cyan());
    println!(
        "  Files skipped:   {} (unchanged)",
        style(result.files_skipped).dim()
    );

    if !result.errors.is_empty() {
        println!("\n{}Errors ({}):", ERROR, result.errors.len());
        for error in result.errors.iter().take(10) {
            println!("  - {}", style(error).red());
        }
        if result.errors.len() > 10 {
            println!("  ... and {} more", result.errors.len() - 10);
        }
    }

    // Prune deleted files
    if prune {
        let pruned = indexer.prune_deleted(path, true).await?;
        if pruned > 0 {
            println!(
                "\n  Pruned:          {} (deleted files removed from index)",
                style(pruned).yellow()
            );
        }
    }

    // Show stats
    let stats = store.stats().await?;
    println!("\n{}Index Statistics:", INFO);
    println!("  Total files:     {}", stats.total_files);
    println!("  Total chunks:    {}", stats.total_chunks);
    println!("  Index size:      {} KB", stats.index_size_bytes / 1024);
    if let Some(updated) = stats.last_updated {
        println!("  Last updated:    {}", updated.format("%Y-%m-%d %H:%M:%S"));
    }

    Ok(())
}

pub async fn run_index_status(path: &Path, output_dir: &str) -> Result<()> {
    let store_path = path.join(output_dir).join("index.json");

    if !store_path.exists() {
        println!("{}No index found at {}", INFO, store_path.display());
        println!("Run `agentlens index` to build the search index.");
        return Ok(());
    }

    let store: Arc<dyn VectorStore> = Arc::new(GobStore::new(store_path.clone()));
    store.load().await?;

    let stats = store.stats().await?;

    println!("\n{}Index Status: {}\n", INFO, store_path.display());
    println!("  Total files:     {}", style(stats.total_files).green());
    println!("  Total chunks:    {}", style(stats.total_chunks).cyan());
    println!(
        "  Index size:      {} KB",
        style(stats.index_size_bytes / 1024).yellow()
    );
    if let Some(updated) = stats.last_updated {
        println!(
            "  Last updated:    {}",
            style(updated.format("%Y-%m-%d %H:%M:%S")).dim()
        );
    }

    Ok(())
}

pub async fn run_index_clear(path: &Path, output_dir: &str) -> Result<()> {
    let store_path = path.join(output_dir).join("index.json");

    if !store_path.exists() {
        println!("{}No index found.", INFO);
        return Ok(());
    }

    let store: Arc<dyn VectorStore> = Arc::new(GobStore::new(store_path));
    store.load().await?;
    store.clear().await?;

    println!("{}Index cleared successfully.", SUCCESS);

    Ok(())
}
