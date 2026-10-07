use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::fmt;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use url::Url;

/// A JSON-RPC error answer, kept typed so a refused rename (an expected
/// outcome) is distinguishable from a broken server.
#[derive(Debug)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
}

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (LSP error {})", self.message, self.code)
    }
}

impl std::error::Error for RpcError {}

/// One stdio language-server conversation with the TypeScript 7 native engine.
/// Requests run strictly one at a time; a reader task drains the server's
/// output so a flood of diagnostics can never block the process.
pub struct Session {
    child: Child,
    stdin: ChildStdin,
    incoming: UnboundedReceiver<Value>,
    next_id: u64,
    versions: HashMap<PathBuf, i64>,
}

impl Session {
    pub async fn start(executable: &Path, root: &Path) -> Result<Self> {
        let mut child = Command::new(executable)
            .args(["--lsp", "--stdio"])
            .current_dir(root)
            .kill_on_drop(true)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| {
                format!(
                    "Could not start the TypeScript native engine {}",
                    executable.display()
                )
            })?;
        let stdin = child
            .stdin
            .take()
            .context("TypeScript engine stdin was not piped")?;
        let stdout = child
            .stdout
            .take()
            .context("TypeScript engine stdout was not piped")?;
        let (sender, incoming) = unbounded_channel();
        tokio::spawn(async move {
            let mut reader = BufReader::new(stdout);
            // A read error or EOF closes the channel; the waiting request reports it.
            while let Ok(message) = read_message(&mut reader).await {
                if sender.send(message).is_err() {
                    break;
                }
            }
        });
        let mut session = Self {
            child,
            stdin,
            incoming,
            next_id: 1,
            versions: HashMap::new(),
        };
        session.initialize(root).await?;
        Ok(session)
    }

    pub fn pid(&self) -> Option<u32> {
        self.child.id()
    }

    async fn initialize(&mut self, root: &Path) -> Result<()> {
        let uri = Url::from_directory_path(root)
            .map_err(|_| anyhow::anyhow!("Invalid project path {}", root.display()))?
            .to_string();
        // rootUri is mandatory for this server; UTF-8 positions keep every
        // offset a plain byte offset in the Rust-side text.
        let init = self
            .request(
                "initialize",
                json!({
                    "processId": std::process::id(),
                    "rootUri": uri,
                    "workspaceFolders": [{ "uri": uri, "name": "project" }],
                    "capabilities": {
                        "general": { "positionEncodings": ["utf-8"] },
                        "textDocument": { "rename": { "prepareSupport": true } },
                    },
                }),
            )
            .await?;
        let capabilities = &init["capabilities"];
        if capabilities["positionEncoding"] != "utf-8" {
            bail!(
                "The TypeScript engine does not support UTF-8 positions: {}",
                capabilities["positionEncoding"]
            );
        }
        if capabilities["renameProvider"].is_null() || capabilities["referencesProvider"].is_null()
        {
            bail!("The TypeScript engine does not offer rename and references");
        }
        self.notify("initialized", json!({})).await
    }

    async fn send(&mut self, message: Value) -> Result<()> {
        let body = serde_json::to_string(&message)?;
        let framed = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
        self.stdin.write_all(framed.as_bytes()).await?;
        self.stdin.flush().await?;
        Ok(())
    }

    pub async fn notify(&mut self, method: &str, params: Value) -> Result<()> {
        self.send(json!({ "jsonrpc": "2.0", "method": method, "params": params }))
            .await
    }

    /// Send a request and wait for its answer. An error answer is returned as
    /// a downcastable [`RpcError`].
    pub async fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        let id = self.next_id;
        self.next_id += 1;
        self.send(json!({ "jsonrpc": "2.0", "id": id, "method": method, "params": params }))
            .await?;
        loop {
            let message = self.incoming.recv().await.with_context(|| {
                format!("The TypeScript engine closed its connection during {method}")
            })?;
            if message.get("method").is_some() {
                if let Some(request_id) = message.get("id") {
                    self.answer_server_request(request_id.clone(), &message)
                        .await?;
                }
                continue; // notifications (logs, diagnostics) carry nothing we need
            }
            if message.get("id") != Some(&json!(id)) {
                continue;
            }
            if let Some(error) = message.get("error") {
                return Err(RpcError {
                    code: error["code"].as_i64().unwrap_or(0),
                    message: error["message"]
                        .as_str()
                        .unwrap_or("unknown error")
                        .to_string(),
                }
                .into());
            }
            return Ok(message["result"].clone());
        }
    }

    async fn answer_server_request(&mut self, id: Value, request: &Value) -> Result<()> {
        let method = request["method"].as_str().unwrap_or_default();
        let reply = match method {
            "client/registerCapability"
            | "client/unregisterCapability"
            | "window/workDoneProgress/create" => {
                json!({ "jsonrpc": "2.0", "id": id, "result": null })
            }
            "workspace/configuration" => {
                let count = request["params"]["items"].as_array().map_or(0, Vec::len);
                json!({ "jsonrpc": "2.0", "id": id, "result": vec![Value::Null; count] })
            }
            _ => {
                json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32601, "message": format!("Unsupported client request {method}") } })
            }
        };
        self.send(reply).await
    }

    /// Show the server a document's content. Opens it on first use and sends a
    /// full-text change afterwards, so verification never touches the disk.
    pub async fn sync_document(&mut self, path: &Path, text: &str) -> Result<()> {
        let uri = Url::from_file_path(path)
            .map_err(|_| anyhow::anyhow!("Invalid document path {}", path.display()))?
            .to_string();
        match self.versions.get(path).copied() {
            None => {
                self.versions.insert(path.to_path_buf(), 1);
                self.notify(
                    "textDocument/didOpen",
                    json!({ "textDocument": { "uri": uri, "languageId": language_id(path), "version": 1, "text": text } }),
                )
                .await
            }
            Some(version) => {
                self.versions.insert(path.to_path_buf(), version + 1);
                self.notify(
                    "textDocument/didChange",
                    json!({ "textDocument": { "uri": uri, "version": version + 1 }, "contentChanges": [{ "text": text }] }),
                )
                .await
            }
        }
    }

    pub async fn shutdown(mut self) {
        let _ = tokio::time::timeout(
            Duration::from_secs(3),
            self.request("shutdown", Value::Null),
        )
        .await;
        let _ = self.notify("exit", Value::Null).await;
        let _ = tokio::time::timeout(Duration::from_secs(2), self.child.wait()).await;
    }
}

fn language_id(path: &Path) -> &'static str {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("tsx") => "typescriptreact",
        Some("jsx") => "javascriptreact",
        Some("js" | "mjs" | "cjs") => "javascript",
        _ => "typescript",
    }
}

async fn read_message(reader: &mut BufReader<ChildStdout>) -> Result<Value> {
    let mut length = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).await? == 0 {
            bail!("Server closed its output");
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':')
            && name.eq_ignore_ascii_case("content-length")
        {
            length = Some(value.trim().parse::<usize>()?);
        }
    }
    let mut body = vec![0; length.context("Message without Content-Length")?];
    reader.read_exact(&mut body).await?;
    Ok(serde_json::from_slice(&body)?)
}
