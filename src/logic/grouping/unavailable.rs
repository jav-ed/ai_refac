//! What to tell the caller when the tool behind a language is not there. A
//! language server is looked for in a fixed order and the answer names every
//! place that was tried and the command (`refac doctor <language>`) that
//! teaches the install, so the caller can fix it without asking anyone.

use crate::servers;
use std::path::Path;

/// The message for a move batch whose driver is not available.
pub fn message(language: &str, project: Option<&Path>) -> String {
    let project = project
        .map(Path::to_path_buf)
        .or_else(|| std::env::current_dir().ok())
        .unwrap_or_default();
    match language {
        // Python moves are done by Rope or Pyrefly, not by the server that
        // `refac doctor python` checks for renames.
        "python" => "Driver for 'python' is not available. refac moves Python files with Rope (`pip install rope`, into the project's .venv or the system python3) or Pyrefly (`pip install pyrefly` into .venv); neither was found. Renaming a Python symbol uses another tool: `refac doctor python`.".to_string(),
        _ => servers::missing_server_message(language, &project)
            .unwrap_or_else(|| format!("Driver for '{language}' is not available.")),
    }
}

#[cfg(test)]
mod tests;
