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

    // 2. Group files by language
    let targets = req.target_path.as_ref().ok_or_else(|| anyhow::anyhow!("Target path required for move"))?;
    
    // Map: Language -> Vec<(Source, Target)>
    let mut batch_map: std::collections::HashMap<String, Vec<(String, String)>> = std::collections::HashMap::new();

    for (src, tgt) in req.source_path.iter().zip(targets.iter()) {
        let path = std::path::Path::new(src);
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        
        let lang = match ext {
            "py" => "python".to_string(),
            "ts" | "tsx" | "js" | "jsx" => "typescript".to_string(),
            "rs" => "rust".to_string(),
            "go" => "go".to_string(),
            "dart" => "dart".to_string(),
            _ => {
                tracing::warn!("Skipping file with unsupported extension: {}", src);
                continue;
            }
        };

        batch_map.entry(lang).or_default().push((src.clone(), tgt.clone()));
    }

    // 3. Dispatch Batches
    let mut results = Vec::new();
    
    for (lang, files) in batch_map {
        let driver = get_driver_by_lang(&lang)?;
        
        if !driver.check_availability().await? {
            bail!("Driver for '{}' is not available.", lang);
        }

        match driver.move_files(files.clone()).await {
            Ok(_) => results.push(format!("Moved {} {} files.", files.len(), lang)),
            Err(e) => results.push(format!("Failed to move {} files: {}", lang, e)),
        }
    }

    Ok(results.join("\n"))
}

fn get_driver_by_lang(lang: &str) -> Result<Box<dyn crate::drivers::RefactorDriver>> {
    let driver: Box<dyn RefactorDriver> = match lang {
        "python" => Box::new(crate::drivers::python::PythonDriver::new()),
        "typescript" => Box::new(crate::drivers::typescript::TypeScriptDriver),
        "rust" => Box::new(crate::drivers::rust::RustDriver::new()),
        "go" => Box::new(crate::drivers::go::GoDriver::new()),
        "dart" => Box::new(crate::drivers::dart::DartDriver::new()),
        _ => bail!("Unsupported language: {}", lang),
    };
    Ok(driver)
}
