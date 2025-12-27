use anyhow::Result;
use rmcp::{
    ServerHandler,
    handler::server::{router::tool::ToolRouter, wrapper::Parameters},
    model::{ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
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

    #[tool(description = "Move/rename file(s) or folder(s) with intelligent reference updating. If project_path is not provided, the server will attempt to auto-detect the project root using the client's 'roots' capability.")]
    async fn refactor(
        &self,
        Parameters(mut req): Parameters<RefactorRequest>,
        context: rmcp::service::RequestContext<rmcp::service::RoleServer>,
    ) -> Result<String, String> {
        tracing::info!("Received refactor request: {:?}", req);
        
        let mut assumed_root: Option<String> = None;

        // Auto-detect project_path if missing
        if req.project_path.is_none() {
            tracing::info!("Refactor request missing project_path. Attempting auto-detection...");
            
            // Check if client supports roots
            // Note: rmcp RequestContext doesn't expose capability checks directly on the struct easily without digging into internals,
            // but we can try to call list_roots and handle the error/empty result.
            // Actually, context has `capabilities` field usually if exposed, let's just try to call list_roots.
            
            match context.peer.list_roots().await {
                Ok(result) => {
                    if let Some(first_root) = result.roots.first() {
                         tracing::info!("Auto-detected root: {:?}", first_root);
                         // Convert URI to path. Assuming file:// convention.
                         // We can use url crate if available, or simple string manipulation for now if we want to avoid extra deps, 
                         // but we saw url in Cargo.toml.
                         if let Ok(url) = url::Url::parse(&first_root.uri) {
                             if let Ok(path) = url.to_file_path() {
                                 if let Some(path_str) = path.to_str() {
                                     let p = path_str.to_string();
                                     assumed_root = Some(p.clone());
                                     req.project_path = Some(p);
                                 }
                             }
                         }
                    } else {
                         tracing::warn!("Client returned empty roots list.");
                    }
                }
                Err(e) => {
                    tracing::warn!("Failed to list roots from client: {:?}", e);
                }
            }
        }

        match handle_refactor(req).await {
            Ok(res) => Ok(res),
            Err(e) => {
                if let Some(root) = assumed_root {
                     // Enhanced error message for assumed root
                     let error_msg = format!(
                         "Refactoring failed using the auto-detected project root: '{}'. If this path is incorrect, please retry the operation providing the explicit 'project_path' argument.\n\nOriginal Error: {}",
                         root, e
                     );
                     Err(error_msg)
                } else {
                    Err(e.to_string())
                }
            }
        }
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
