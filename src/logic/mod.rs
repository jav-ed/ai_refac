use anyhow::{Result, bail};
use crate::validation::initial_sanity_check;
use crate::drivers::RefactorDriver;

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

    // 2. Dispatch
    let source_file = &req.source_path[0];
    let driver = get_driver_for_file(source_file)?;
    
    // Check availability
    if !driver.check_availability().await? {
        bail!("Driver for '{}' is not available in this environment.", driver.lang());
    }

    match req.operation.as_str() {
        "move" => {
             let targets = req.target_path.as_ref().ok_or_else(|| anyhow::anyhow!("Target path required for move"))?;
             // For now, handle single file move. Loop for multiple.
             for (src, tgt) in req.source_path.iter().zip(targets.iter()) {
                 driver.move_file(src, tgt).await?;
             }
             Ok(format!("Successfully moved {} files.", req.source_path.len()))
        },
        _ => bail!("Unsupported operation: {}", req.operation),
    }
}

fn get_driver_for_file(path_str: &str) -> Result<Box<dyn crate::drivers::RefactorDriver>> {
    let path = std::path::Path::new(path_str);
    let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
    
    let lang = match ext {
        "py" => "python",
        "ts" | "tsx" | "js" | "jsx" => "typescript",
        "rs" => "rust",
        _ => bail!("No refactoring driver found for extension: .{}", ext),
    };

    let driver: Box<dyn RefactorDriver> = match lang {
        "python" => Box::new(crate::drivers::python::PythonDriver::new()),
        "typescript" => Box::new(crate::drivers::typescript::TypeScriptDriver),
        "rust" => Box::new(crate::drivers::rust::RustDriver::new()),
        _ => bail!("Unsupported language: {}", lang), // This case should ideally not be reached if `lang` is derived correctly
    };
    Ok(driver)
}
