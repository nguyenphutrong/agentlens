use anyhow::{Context, Result};
use rmcp::{transport::stdio, ServiceExt};
use std::path::Path;

use crate::cli::Args;
use crate::mcp::AgentmapServer;

pub async fn run_mcp_server(args: &Args, work_path: &Path) -> Result<()> {
    let output_path = if args.output.is_absolute() {
        args.output.clone()
    } else {
        work_path.join(&args.output)
    };

    let server = AgentmapServer::new(work_path.to_path_buf(), output_path, args.clone());

    eprintln!("Starting agentmap MCP server (stdio)...");
    eprintln!("Work path: {}", work_path.display());

    let service = server
        .serve(stdio())
        .await
        .context("Failed to start MCP server")?;

    service.waiting().await.context("MCP server error")?;

    Ok(())
}

pub async fn run_mcp_http_server(args: &Args, work_path: &Path, port: u16) -> Result<()> {
    let output_path = if args.output.is_absolute() {
        args.output.clone()
    } else {
        work_path.join(&args.output)
    };

    let _server = AgentmapServer::new(work_path.to_path_buf(), output_path, args.clone());

    eprintln!(
        "Starting agentmap MCP server (HTTP/SSE on port {})...",
        port
    );
    eprintln!("Work path: {}", work_path.display());

    anyhow::bail!("HTTP/SSE transport not yet implemented. Use stdio mode: agentmap serve --mcp")
}
