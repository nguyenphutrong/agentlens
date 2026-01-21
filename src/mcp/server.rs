use rmcp::{
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{
        AnnotateAble, CallToolResult, Content, ListResourcesResult, PaginatedRequestParam,
        RawResource, ReadResourceRequestParam, ReadResourceResult, ResourceContents,
        ServerCapabilities, ServerInfo,
    },
    service::RequestContext,
    tool, tool_handler, tool_router, ErrorData as McpError, RoleServer, ServerHandler,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::analyze::extract_symbols;
use crate::cli::check::check_staleness;
use crate::cli::Args;
use crate::config::Config;
use crate::scan::scan_directory;
use crate::search::{create_embedder, EmbedderConfig, GobStore, Searcher};
use crate::types::{Symbol, Visibility};

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetModuleParams {
    #[schemars(description = "Module slug (e.g., 'src-analyze' or 'src-cli')")]
    pub slug: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct GetOutlineParams {
    #[schemars(description = "Relative file path (e.g., 'src/main.rs')")]
    pub file: String,
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
pub struct SemanticSearchParams {
    #[schemars(
        description = "Natural language search query (e.g., 'authentication flow', 'error handling')"
    )]
    pub query: String,
    #[schemars(description = "Maximum number of results to return (default: 10)")]
    pub limit: Option<usize>,
    #[schemars(
        description = "Enable hybrid search combining vector and text matching (default: true)"
    )]
    pub hybrid: Option<bool>,
}

#[derive(Clone)]
pub struct AgentlensServer {
    work_path: Arc<PathBuf>,
    output_path: Arc<PathBuf>,
    args: Arc<RwLock<Args>>,
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl AgentlensServer {
    pub fn new(work_path: PathBuf, output_path: PathBuf, args: Args) -> Self {
        Self {
            work_path: Arc::new(work_path),
            output_path: Arc::new(output_path),
            args: Arc::new(RwLock::new(args)),
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Regenerate agentlens documentation for the codebase")]
    async fn regenerate(&self) -> Result<CallToolResult, McpError> {
        let mut args = self.args.write().await;
        args.force = true;

        match crate::runner::run_analysis(&args, &self.work_path) {
            Ok(()) => Ok(CallToolResult::success(vec![Content::text(
                "Documentation regenerated successfully",
            )])),
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Regeneration failed: {}",
                e
            ))])),
        }
    }

    #[tool(description = "Get module documentation by slug (e.g., 'src-analyze')")]
    async fn get_module(
        &self,
        Parameters(params): Parameters<GetModuleParams>,
    ) -> Result<CallToolResult, McpError> {
        let slug = &params.slug;
        let module_dir = self.output_path.join("modules").join(slug);

        if !module_dir.exists() {
            return Err(McpError::invalid_params(
                format!("Module '{}' not found", slug),
                Some(json!({ "slug": slug })),
            ));
        }

        let mut content = String::new();

        let module_md_path = module_dir.join("MODULE.md");
        if module_md_path.exists() {
            if let Ok(text) = fs::read_to_string(&module_md_path) {
                content.push_str(&text);
                content.push_str("\n\n---\n\n");
            }
        }

        for file in &["outline.md", "memory.md", "imports.md"] {
            let file_path = module_dir.join(file);
            if file_path.exists() {
                if let Ok(text) = fs::read_to_string(&file_path) {
                    if !text.is_empty() {
                        content.push_str(&format!("## {}\n\n", file));
                        content.push_str(&text);
                        content.push_str("\n\n");
                    }
                }
            }
        }

        if content.is_empty() {
            return Err(McpError::invalid_params(
                format!("Module '{}' has no content", slug),
                Some(json!({ "slug": slug })),
            ));
        }

        Ok(CallToolResult::success(vec![Content::text(content)]))
    }

    #[tool(description = "Check if documentation is stale and needs regeneration")]
    async fn check_stale(&self) -> Result<CallToolResult, McpError> {
        let args = self.args.read().await;

        match check_staleness(&args, &self.work_path) {
            Ok(result) => {
                let response = json!({
                    "is_stale": result.is_stale,
                    "stale_modules": result.stale_modules,
                    "new_modules": result.new_modules,
                    "removed_modules": result.removed_modules,
                });
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&response).unwrap_or_default(),
                )]))
            }
            Err(e) => Ok(CallToolResult::success(vec![Content::text(format!(
                "Check failed: {}",
                e
            ))])),
        }
    }

    #[tool(description = "Get symbol outline for a specific file")]
    async fn get_outline(
        &self,
        Parameters(params): Parameters<GetOutlineParams>,
    ) -> Result<CallToolResult, McpError> {
        let file = &params.file;
        let full_path = self.work_path.join(file);

        if !full_path.exists() {
            return Err(McpError::invalid_params(
                format!("File '{}' not found", file),
                Some(json!({ "file": file })),
            ));
        }

        let args = self.args.read().await;
        let max_depth = if args.depth > 0 {
            Some(args.depth)
        } else {
            None
        };

        let files = scan_directory(
            &self.work_path,
            args.threshold,
            !args.no_gitignore,
            max_depth,
        )
        .map_err(|e| McpError::internal_error(format!("Scan failed: {}", e), None))?;

        let file_entry = files.iter().find(|f| &f.relative_path == file);

        match file_entry {
            Some(entry) => {
                let content = fs::read_to_string(&full_path)
                    .map_err(|e| McpError::internal_error(format!("Read failed: {}", e), None))?;

                let symbols: Vec<Symbol> = extract_symbols(entry, &content);

                let outline = format_symbols_as_outline(file, &symbols);
                Ok(CallToolResult::success(vec![Content::text(outline)]))
            }
            None => Err(McpError::invalid_params(
                format!("File '{}' not in scan results", file),
                Some(json!({ "file": file })),
            )),
        }
    }

    #[tool(description = "Semantic search across the codebase using natural language queries")]
    async fn semantic_search(
        &self,
        Parameters(params): Parameters<SemanticSearchParams>,
    ) -> Result<CallToolResult, McpError> {
        let query = &params.query;
        let limit = params.limit.unwrap_or(10);
        let hybrid = params.hybrid.unwrap_or(true);

        let config = Config::load(&self.work_path);
        let search_config = config.and_then(|c| c.search).unwrap_or_default();

        let embedder_config = EmbedderConfig {
            provider: search_config.embedder.provider.clone(),
            model: search_config.embedder.model.clone(),
            endpoint: search_config.embedder.endpoint.clone(),
            dimensions: search_config.embedder.dimensions,
        };
        let embedder = Arc::from(create_embedder(&embedder_config));

        let index_path = self.output_path.join("index.json");
        let store = Arc::new(GobStore::new(index_path));

        let searcher = Searcher::new(store, embedder, hybrid, search_config.search.hybrid_k);

        let results = searcher
            .smart_search(query, limit)
            .await
            .map_err(|e| McpError::internal_error(format!("Search failed: {}", e), None))?;

        if results.is_empty() {
            return Ok(CallToolResult::success(vec![Content::text(
                "No results found. Make sure the index exists (run `agentlens index` first).",
            )]));
        }

        let formatted: Vec<serde_json::Value> = results
            .iter()
            .map(|r| {
                json!({
                    "file": r.chunk.file_path,
                    "score": format!("{:.3}", r.score),
                    "lines": format!("{}-{}", r.chunk.start_line, r.chunk.end_line),
                    "type": format!("{:?}", r.chunk.chunk_type),
                    "content_preview": r.chunk.content.chars().take(200).collect::<String>(),
                })
            })
            .collect();

        let response = json!({
            "query": query,
            "result_count": results.len(),
            "results": formatted,
        });

        Ok(CallToolResult::success(vec![Content::text(
            serde_json::to_string_pretty(&response).unwrap_or_default(),
        )]))
    }
}

fn format_symbols_as_outline(file_path: &str, symbols: &[Symbol]) -> String {
    if symbols.is_empty() {
        return format!("# {}\n\nNo symbols found.", file_path);
    }

    let mut output = format!("# {}\n\n", file_path);
    output.push_str("| Line | Kind | Name | Visibility |\n");
    output.push_str("| ---- | ---- | ---- | ---------- |\n");

    for sym in symbols {
        let visibility = match sym.visibility {
            Visibility::Public => "pub",
            Visibility::Private => "(private)",
            Visibility::Protected => "(protected)",
            Visibility::Internal => "(internal)",
        };
        output.push_str(&format!(
            "| {} | {} | {} | {} |\n",
            sym.line_range.start, sym.kind, sym.name, visibility
        ));
    }

    output
}

#[tool_handler]
impl ServerHandler for AgentlensServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            capabilities: ServerCapabilities::builder()
                .enable_tools()
                .enable_resources()
                .build(),
            server_info: rmcp::model::Implementation {
                name: "agentlens".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
                ..Default::default()
            },
            instructions: Some(
                "Agentlens MCP server - query and regenerate codebase documentation".to_string(),
            ),
            ..Default::default()
        }
    }

    async fn list_resources(
        &self,
        _request: Option<PaginatedRequestParam>,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ListResourcesResult, McpError> {
        Ok(ListResourcesResult {
            resources: vec![
                RawResource::new("agentlens://index", "INDEX.md".to_string()).no_annotation(),
            ],
            ..Default::default()
        })
    }

    async fn read_resource(
        &self,
        request: ReadResourceRequestParam,
        _ctx: RequestContext<RoleServer>,
    ) -> Result<ReadResourceResult, McpError> {
        match request.uri.as_str() {
            "agentlens://index" => {
                let index_path = self.output_path.join("INDEX.md");
                let content = fs::read_to_string(&index_path).map_err(|e| {
                    McpError::resource_not_found(
                        format!("INDEX.md not found: {}", e),
                        Some(json!({ "uri": request.uri })),
                    )
                })?;
                Ok(ReadResourceResult {
                    contents: vec![ResourceContents::text(content, request.uri)],
                })
            }
            _ => Err(McpError::resource_not_found(
                "Unknown resource",
                Some(json!({ "uri": request.uri })),
            )),
        }
    }
}
