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
            if !dir_looks_like_typescript(&resolved) {
                bail!(
                    "Directory moves are only supported for TypeScript/JavaScript projects. \
                     '{}' does not appear to contain TypeScript or JavaScript files.",
                    src
                );
            }
            batch_map
                .entry("typescript".to_string())
                .or_default()
                .push((src.clone(), tgt.clone()));
            continue;
        }

        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");

        let lang = match ext {
            "md" => "markdown".to_string(),
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

    // 3. Dispatch Batches — sorted for deterministic output order
    let root = req.project_path.as_ref().map(std::path::Path::new);
    let mut successful_files: std::collections::HashMap<String, Vec<(String, String)>> =
        std::collections::HashMap::new();
    // (lang, attempted files, error message)
    let mut failed_batches: Vec<(String, Vec<(String, String)>, String)> = Vec::new();

    let mut dispatch_order: Vec<(String, Vec<(String, String)>)> = batch_map.into_iter().collect();
    dispatch_order.sort_by(|a, b| a.0.cmp(&b.0));

    for (lang, files) in dispatch_order {
        let driver = get_driver_by_lang(&lang)?;

        if !driver.check_availability().await? {
            bail!("Driver for '{}' is not available.", lang);
        }

        match driver.move_files(files.clone(), root).await {
            Ok(_) => {
                successful_files.insert(lang, files);
            }
            Err(e) => failed_batches.push((lang, files, e.to_string())),
        }
    }

    // If everything failed, bail with a structured error rather than a blank response.
    if !failed_batches.is_empty() && successful_files.is_empty() && skipped_files.is_empty() {
        let lines: Vec<String> = failed_batches
            .iter()
            .map(|(lang, files, err)| {
                let file_list = files
                    .iter()
                    .map(|(s, t)| format!("  {} -> {}", rel_display(s, root), rel_display(t, root)))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("{}: {}\n{}", capitalize(lang), err, file_list)
            })
            .collect();
        bail!("{}", lines.join("\n\n"));
    }

    // 4. Build response
    let total_files: usize = successful_files.values().map(|v| v.len()).sum();
    let mut response = format!(
        "// Alhamdulillah {} file{} successfully refactored:\n",
        total_files,
        if total_files == 1 { " was" } else { " were" }
    );

    let mut success_langs: Vec<_> = successful_files.keys().cloned().collect();
    success_langs.sort();

    for lang in success_langs {
        let files = &successful_files[&lang];
        response.push_str(&format!("\n// {} results:\n\n", capitalize(&lang)));
        for (src, tgt) in files {
            response.push_str(&format!(
                "{} -> {}  \n",
                rel_display(src, root),
                rel_display(tgt, root)
            ));
        }

        // For Go: report any files gopls moved collaterally beyond what was requested.
        if lang == "go" {
            let collaterals = detect_go_collaterals(files, root);
            if !collaterals.is_empty() {
                response.push_str(
                    "\n// Note — Go moves entire packages. \
                     The following files were also relocated as part of the package rename:  \n\n",
                );
                for path in collaterals {
                    response.push_str(&format!("{}  \n", rel_display(&path.display().to_string(), root)));
                }
            }
        }
    }

    if !failed_batches.is_empty() {
        response.push_str("\n// Failed:\n");
        for (lang, files, err) in &failed_batches {
            response.push_str(&format!("\n// {} — {}\n\n", capitalize(lang), err));
            for (src, tgt) in files {
                response.push_str(&format!(
                    "{} -> {}  \n",
                    rel_display(src, root),
                    rel_display(tgt, root)
                ));
            }
        }
    }

    if !skipped_files.is_empty() {
        response.push_str("\n// Skipped (unsupported extension):  \n\n");
        for file in &skipped_files {
            response.push_str(&format!("{}  \n", rel_display(file, root)));
        }
    }

    Ok(response)
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

fn rel_display(path: &str, root: Option<&std::path::Path>) -> String {
    if let Some(r) = root {
        std::path::Path::new(path)
            .strip_prefix(r)
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| path.to_string())
    } else {
        path.to_string()
    }
}

/// After a successful Go batch move, scan the target directories for .go files
/// that were not in the requested file map — these are collateral moves performed
/// by gopls as part of its package-level rename.
fn detect_go_collaterals(
    file_map: &[(String, String)],
    root: Option<&std::path::Path>,
) -> Vec<std::path::PathBuf> {
    // Build the set of requested target absolute paths.
    let requested: std::collections::HashSet<std::path::PathBuf> = file_map
        .iter()
        .map(|(_, tgt)| {
            let p = std::path::Path::new(tgt);
            if p.is_absolute() {
                p.to_path_buf()
            } else if let Some(r) = root {
                r.join(p)
            } else {
                p.to_path_buf()
            }
        })
        .collect();

    // Collect the unique target directories.
    let target_dirs: std::collections::HashSet<std::path::PathBuf> = requested
        .iter()
        .filter_map(|p| p.parent().map(|d| d.to_path_buf()))
        .collect();

    // Any .go file in those directories that was not explicitly requested is collateral.
    let mut collaterals = Vec::new();
    for dir in &target_dirs {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("go")
                    && !requested.contains(&path)
                {
                    collaterals.push(path);
                }
            }
        }
    }
    collaterals.sort();
    collaterals
}

fn get_driver_by_lang(lang: &str) -> Result<Box<dyn crate::drivers::RefactorDriver>> {
    let driver: Box<dyn RefactorDriver> = match lang {
        "markdown" => Box::new(crate::drivers::markdown::MarkdownDriver::new()),
        "python" => Box::new(crate::drivers::python::PythonDriver::new()),
        "typescript" => Box::new(crate::drivers::typescript::TypeScriptDriver),
        "rust" => Box::new(crate::drivers::rust::RustDriver::new()),
        "go" => Box::new(crate::drivers::go::GoDriver::new()),
        "dart" => Box::new(crate::drivers::dart::DartDriver::new()),
        _ => bail!("Unsupported language: {}", lang),
    };
    Ok(driver)
}

fn dir_looks_like_typescript(dir: &std::path::Path) -> bool {
    std::fs::read_dir(dir)
        .ok()
        .map(|entries| {
            entries.flatten().any(|entry| {
                entry
                    .path()
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| matches!(e, "ts" | "tsx" | "js" | "jsx"))
                    .unwrap_or(false)
            })
        })
        .unwrap_or(false)
}
