use crate::drivers::RefactorDriver;
use crate::validation::initial_sanity_check;
use anyhow::{Result, bail};

/// Parameters for a refactoring request.
#[derive(Debug, serde::Deserialize, schemars::JsonSchema)]
pub struct RefactorRequest {
    #[schemars(
        description = "List of source paths. Use paths relative to project_path (recommended). Example: `Src/Features/Auth/login_Service.ts`."
    )]
    pub source_path: Vec<String>,
    #[schemars(
        description = "List of target paths (1:1 mapping with source_path). Use the same path style as source_path and keep them relative to project_path."
    )]
    pub target_path: Option<Vec<String>>,
    #[schemars(description = "Type of operation (currently only 'move' is supported)")]
    pub operation: String,
    #[schemars(
        description = "Absolute path to the language package root (not monorepo root). For TypeScript/JS, this should usually be the folder containing the relevant `tsconfig.json`."
    )]
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
    initial_sanity_check(
        &req.source_path,
        &req.operation,
        req.target_path.as_ref(),
        req.project_path.as_deref(),
    )?;

    // 2. Group files by language
    let targets = req
        .target_path
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("Target path required for move"))?;

    // Map: Language -> Vec<(Source, Target)>
    let mut batch_map: std::collections::HashMap<String, Vec<(String, String)>> =
        std::collections::HashMap::new();
    let mut skipped_files = Vec::new();

    for (src, tgt) in req.source_path.iter().zip(targets.iter()) {
        let path = std::path::Path::new(src);

        // Resolve the path so we can check whether it's a directory.
        let resolved = if path.is_absolute() {
            path.to_path_buf()
        } else if let Some(root) = req.project_path.as_deref() {
            std::path::Path::new(root).join(path)
        } else {
            path.to_path_buf()
        };

        if resolved.is_dir() {
            // Directory moves are handled by the TypeScript driver (ts-morph directory.move()).
            // Other language backends operate file-by-file and don't support directory moves.
            batch_map
                .entry("typescript".to_string())
                .or_default()
                .push((src.clone(), tgt.clone()));
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let lang = match ext {
            "py" => "python".to_string(),
            "ts" | "tsx" | "js" | "jsx" => "typescript".to_string(),
            "rs" => "rust".to_string(),
            "go" => "go".to_string(),
            "dart" => "dart".to_string(),
            _ => {
                tracing::warn!("Skipping file with unsupported extension: {}", src);
                skipped_files.push(src.clone());
                continue;
            }
        };

        batch_map
            .entry(lang)
            .or_default()
            .push((src.clone(), tgt.clone()));
    }

    // 3. Dispatch Batches
    let mut total_files = 0;
    let mut successful_files: std::collections::HashMap<String, Vec<(String, String)>> =
        std::collections::HashMap::new();
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
            }
            Err(e) => errors.push(format!("Failed to move {} files: {}", lang, e)),
        }
    }

    if !errors.is_empty() && successful_files.is_empty() {
        bail!("{}", errors.join("\n"));
    }

    let mut response = format!(
        "// Alhamdulillah {} files were successfully refactored like shown below:\n",
        total_files
    );

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
                std::path::Path::new(src)
                    .strip_prefix(r)
                    .unwrap_or(std::path::Path::new(src))
                    .display()
                    .to_string()
            } else {
                src.clone()
            };

            let tgt_disp = if let Some(r) = root {
                std::path::Path::new(tgt)
                    .strip_prefix(r)
                    .unwrap_or(std::path::Path::new(tgt))
                    .display()
                    .to_string()
            } else {
                tgt.clone()
            };

            response.push_str(&format!("{} -> {}  \n", src_disp, tgt_disp));
        }
    }

    if !skipped_files.is_empty() {
        response
            .push_str("\n// Following files were not refactored (unsupported extension):  \n\n");
        for file in skipped_files {
            // Try to display relative path for skipped files too
            let file_disp = if let Some(r) = req.project_path.as_ref().map(std::path::Path::new) {
                std::path::Path::new(&file)
                    .strip_prefix(r)
                    .unwrap_or(std::path::Path::new(&file))
                    .display()
                    .to_string()
            } else {
                file.clone()
            };
            response.push_str(&format!("{}  \n", file_disp));
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
