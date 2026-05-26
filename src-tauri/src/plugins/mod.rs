pub mod sandbox;

use async_trait::async_trait;
use serde_json::Value;
use crate::error::AegisResult;

#[derive(Debug, Clone)]
pub struct PluginInfo {
    pub name: &'static str,
    pub version: &'static str,
    pub description: &'static str,
}

/// Trait every pluggable tool module must implement.
///
/// Plugins are registered at startup via PluginRegistry and dispatched
/// by name. No dynamic linking — all plugins are compiled into the binary.
#[async_trait]
pub trait ToolPlugin: Send + Sync + 'static {
    fn info(&self) -> PluginInfo;
    async fn execute(&self, params: Value) -> AegisResult<Value>;
}
