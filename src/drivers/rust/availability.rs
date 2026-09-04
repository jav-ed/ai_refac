use anyhow::Result;

pub fn rust_analyzer_command() -> String {
    if which::which("rust-analyzer").is_ok() {
        return "rust-analyzer".to_string();
    }

    if let Some(home) = std::env::var_os("HOME") {
        let cargo_binary = std::path::Path::new(&home).join(".cargo/bin/rust-analyzer");
        if cargo_binary.exists() {
            return cargo_binary.to_string_lossy().into_owned();
        }
    }

    "rust-analyzer".to_string()
}

pub async fn rust_analyzer_is_available() -> Result<bool> {
    let command = rust_analyzer_command();
    let output = tokio::process::Command::new(&command)
        .arg("--version")
        .output()
        .await;

    match output {
        Ok(output) => Ok(output.status.success()),
        Err(error) => {
            tracing::warn!(%error, %command, "rust-analyzer availability check failed");
            Ok(false)
        }
    }
}
