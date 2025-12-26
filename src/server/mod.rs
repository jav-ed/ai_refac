use anyhow::Result;
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    schemars, tool, tool_handler, tool_router,
};
use crate::logic::{RefactorRequest, handle_refactor};

#[derive(Debug, Clone)]
pub struct RefactorServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl RefactorServer {
    pub fn new() -> Self {
        Self {
            tool_router: Self::tool_router(),
        }
    }

    #[tool(description = "Move/rename file(s) or folder(s) with intelligent reference updating.")]
    async fn refactor(&self, Parameters(req): Parameters<RefactorRequest>) -> Result<String, String> {
        tracing::info!("Received refactor request: {:?}", req);
        
        handle_refactor(req).await.map_err(|e| e.to_string())
    }
}

#[tool_handler]
impl ServerHandler for RefactorServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            server_info: rmcp::model::Implementation {
                name: "refac_mcp".into(),
                version: "0.1.0".into(),
                ..Default::default()
            },
            instructions: Some("A Refactoring MCP Server".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            ..Default::default()
        }
    }
}
