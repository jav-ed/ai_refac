use anyhow::{Result, Context};
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use lsp_types::{
    ClientCapabilities, InitializeParams, RenameFilesParams,
    FileRename, WorkspaceEdit, Uri,
};
use url::Url;
use std::str::FromStr;
use serde_json::Value;

pub struct LspClient {
    binary_path: String,
}

impl LspClient {
    pub fn new(binary_path: &str) -> Self {
        Self {
            binary_path: binary_path.to_string(),
        }
    }

    pub async fn check_availability(&self) -> Result<bool> {
        Ok(std::path::Path::new(&self.binary_path).exists())
    }

    pub async fn initialize_and_rename_files(
        &self,
        args: &[&str],
        file_map: Vec<(String, String)>,
        root_path: Option<&std::path::Path>,
    ) -> Result<()> {
        // 1. Start LSP Process
        let mut child = Command::new(&self.binary_path)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let mut stdin = child.stdin.take().context("Failed to open stdin")?;
        let stdout = child.stdout.take().context("Failed to open stdout")?;
        let mut reader = BufReader::new(stdout);

        // 2. Initialize
        let root_dir = if let Some(r) = root_path {
            r.to_path_buf()
        } else {
            std::env::current_dir()?
        };
        
        let root_uri = Url::from_directory_path(&root_dir).map_err(|_| anyhow::anyhow!("Invalid root path"))?;

        #[allow(deprecated)]
        let init_params = InitializeParams {
            process_id: Some(std::process::id()),
            // root_uri is deprecated, use workspace_folders
            root_uri: None, // Explicitly set to None to avoid warning if we were using it
            workspace_folders: Some(vec![lsp_types::WorkspaceFolder {
                uri: Uri::from_str(root_uri.as_str()).map_err(|e| anyhow::anyhow!("Invalid URI: {}", e))?,
                name: "project".to_string(),
            }]),
            capabilities: ClientCapabilities::default(), // Minimal capabilities
            ..Default::default()
        };

        Self::send_request(&mut stdin, "initialize", 1, init_params).await?;
        let _init_resp = Self::read_response(&mut reader).await?;

        Self::send_request(&mut stdin, "initialized", 2, serde_json::json!({})).await?;

        // 3. Send workspace/willRenameFiles
        let mut file_renames = Vec::new();
        for (source, target) in file_map {
             let source_abs = root_dir.join(source);
             let target_abs = root_dir.join(target);
             
             let source_uri = Url::from_file_path(&source_abs).map_err(|_| anyhow::anyhow!("Invalid source path"))?;
             let target_uri = Url::from_file_path(&target_abs).map_err(|_| anyhow::anyhow!("Invalid target path"))?;
             
             file_renames.push(FileRename {
                 old_uri: source_uri.to_string(),
                 new_uri: target_uri.to_string(),
             });
        }

        let rename_params = RenameFilesParams {
            files: file_renames,
        };

        Self::send_request(&mut stdin, "workspace/willRenameFiles", 3, rename_params).await?;
        let resp = Self::read_response(&mut reader).await?;

        // 4. Apply Edits
        if let Some(result) = resp.get("result") {
             if let Ok(edit) = serde_json::from_value::<WorkspaceEdit>(result.clone()) {
                 apply_workspace_edit(edit).await?;
             }
        }

        // 5. Shutdown (Graceful)
        let _ = Self::send_request(&mut stdin, "shutdown", 4, ());
        let _ = child.kill().await;

        Ok(())
    }

    async fn send_request<T: serde::Serialize>(
        stdin: &mut tokio::process::ChildStdin,
        method: &str,
        id: u64,
        params: T,
    ) -> Result<()> {
        let params_json = serde_json::to_value(params)?;
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params_json,
        });
        let body = serde_json::to_string(&request)?;
        let message = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
        stdin.write_all(message.as_bytes()).await?;
        stdin.flush().await?;
        Ok(())
    }

    async fn read_response(reader: &mut BufReader<tokio::process::ChildStdout>) -> Result<Value> {
        let mut content_length = 0;
        loop {
            let mut line = String::new();
            reader.read_line(&mut line).await?;
            if line == "\r\n" {
                break;
            }
            if line.starts_with("Content-Length: ") {
                content_length = line.trim_start_matches("Content-Length: ").trim().parse()?;
            }
        }

        if content_length == 0 {
            anyhow::bail!("Missing Content-Length header or zero length");
        }

        let mut body_buf = vec![0; content_length];
        reader.read_exact(&mut body_buf).await?;
        let body = String::from_utf8(body_buf)?;
        let response: Value = serde_json::from_str(&body)?;
        Ok(response)
    }
}

async fn apply_workspace_edit(edit: WorkspaceEdit) -> Result<()> {
    if let Some(changes) = edit.changes {
        for (uri, edits) in changes {
            let url = Url::parse(&uri.to_string()).map_err(|e| anyhow::anyhow!("Invalid URI: {}", e))?;
            let path_buf = url.to_file_path().map_err(|_| anyhow::anyhow!("Cannot convert URI to file path"))?;
            
            if !path_buf.exists() {
                 tracing::warn!("LSP returned edit for non-existent file: {:?}", path_buf);
                 continue;
            }

            let mut content = tokio::fs::read_to_string(&path_buf).await?;
            apply_text_edits(&path_buf, &mut content, edits).await?;
            // In a real scenario, we would save the content back.
            // For now, logging only as per previous implementation strategy.
            // tokio::fs::write(&path_buf, content).await?;
        }
    }
    Ok(())
}

async fn apply_text_edits(path: &std::path::Path, content: &mut String, mut edits: Vec<lsp_types::TextEdit>) -> Result<()> {
    edits.sort_by(|a, b| b.range.start.partial_cmp(&a.range.start).unwrap());
    
    // Simplistic line mapping (same as before)
    let mut line_offsets = Vec::new();
    let mut current_offset = 0;
    for line in content.split_inclusive('\n') {
        line_offsets.push(current_offset);
        current_offset += line.len();
    }
    line_offsets.push(current_offset); 

    for edit in edits {
        let start_line = edit.range.start.line as usize;
        
        if start_line >= line_offsets.len() { continue; }
        
        tracing::info!("Would apply edit to {:?}: Replace {:?} with '{}'", path, edit.range, edit.new_text);
    }
    Ok(())
}
