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
    #[schemars(description = "Absolute path to the project root. Required if source/target paths are relative.")]
    pub project_path: Option<String>,
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
    initial_sanity_check(&req.source_path, &req.operation, req.target_path.as_ref(), req.project_path.as_deref())?;

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
    let mut total_files = 0;
    let mut successful_files: std::collections::HashMap<String, Vec<(String, String)>> = std::collections::HashMap::new();
    let mut errors = Vec::new();
    
    for (lang, files) in batch_map {
        let driver = get_driver_by_lang(&lang)?;
        
        if !driver.check_availability().await? {
            bail!("Driver for '{}' is not available.", lang);
        }

        let root = req.project_path.as_ref().map(std::path::Path::new);
        match driver.move_files(files.clone(), root).await {
            Ok(_) => {
                total_files += files.len();
                successful_files.insert(lang, files);
            },
            Err(e) => errors.push(format!("Failed to move {} files: {}", lang, e)),
        }
    }

    if !errors.is_empty() && successful_files.is_empty() {
        bail!("{}", errors.join("\n"));
    }

    let mut response = format!("// Alhamdulillah {} files were successfully refactored like shown below:\n", total_files);
    
    // Sort languages for consistent output
    let mut langs: Vec<_> = successful_files.keys().cloned().collect();
    langs.sort();

    for lang in langs {
        let files = &successful_files[&lang];
        let root = req.project_path.as_ref().map(std::path::Path::new);
        
        let lang_display = if lang.len() > 0 {
            let mut c = lang.chars();
            c.next().unwrap().to_uppercase().collect::<String>() + c.as_str()
        } else {
            lang.clone()
        };

        response.push_str(&format!("\n// {} results:\n\n", lang_display));
        for (src, tgt) in files {
            let src_disp = if let Some(r) = root {
                std::path::Path::new(src).strip_prefix(r).unwrap_or(std::path::Path::new(src)).display().to_string()
            } else {
                src.clone()
            };
            
            let tgt_disp = if let Some(r) = root {
                std::path::Path::new(tgt).strip_prefix(r).unwrap_or(std::path::Path::new(tgt)).display().to_string()
            } else {
                tgt.clone()
            };

            response.push_str(&format!("{} -> {}  \n", src_disp, tgt_disp));
        }
    }

    if !errors.is_empty() {
        response.push_str("\n// --- Errors ---  \n");
        response.push_str(&errors.join("  \n"));
    }

    Ok(response)
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
