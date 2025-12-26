use anyhow::{Result, bail};
use crate::validation::initial_sanity_check;

/// Parameters for a refactoring request.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RefactorRequest {
    #[schemars(description = "List of source file paths (relative to project root)")]
    pub source_path: Vec<String>,
    #[schemars(description = "List of target file paths (1:1 mapping with sources)")]
    pub target_path: Option<Vec<String>>,
    #[schemars(description = "Type of operation (currently only 'move' is supported)")]
    pub operation: String,
}

/// Central entry point for handling refactor requests.
/// 
/// # Internal Docs
/// This function acts as the **Orchestrator**.
/// 1. Runs validation.
/// 2. Determines which driver to use (TODO).
/// 3. Dispatches the request.
pub async fn handle_refactor(req: RefactorRequest) -> Result<String> {
    
    // 1. Validation
    initial_sanity_check(&req.source_path, &req.operation, req.target_path.as_ref())?;

    // 2. Dispatch (Currently stubbed)
    // TODO: Detect language from extensions in `source_path`
    // TODO: Select correct driver from `drivers` module
    
    match req.operation.as_str() {
        "move" => {
             // Mock response for now
             let targets = req.target_path.unwrap_or_default();
             Ok(format!("Validated 'move' operation for {:?} -> {:?}", req.source_path, targets))
        },
        _ => bail!("Unsupported operation: {}", req.operation),
    }
}
