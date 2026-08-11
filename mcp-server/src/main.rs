mod sanitizer;
mod tools;

use rmcp::{ServiceExt, transport::stdio};
use tools::AegisToolService;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Log to stderr so stdout stays clean for JSON-RPC
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "aegisforge_mcp=info,rmcp=warn".into()),
        )
        .with_writer(std::io::stderr)
        .init();

    tracing::info!("AegisForge MCP server starting — stdio transport");

    let service = AegisToolService::new().serve(stdio()).await?;
    service.waiting().await?;

    tracing::info!("MCP server shut down cleanly");
    Ok(())
}
