//! One stdio language-server conversation, shared by the backends that drive a
//! server directly (TypeScript rename, Kotlin). A reader task drains the
//! server's output so a flood of logs can never block the process, requests
//! run strictly one at a time, and the server's own requests are answered
//! while we wait, so a server request that reuses one of our request ids is
//! never mistaken for an answer.

use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::time::{Duration, Instant};
use tokio::io::{AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, Command};
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};
use url::Url;

mod protocol;

pub use protocol::RpcError;
use protocol::read_message;

pub struct SessionConfig<'a> {
    /// Name used in error messages, for example "TypeScript native engine".
    pub name: &'a str,
    pub executable: &'a Path,
    pub args: Vec<String>,
    /// Working directory of the server process.
    pub cwd: &'a Path,
    /// Workspace root announced in `initialize`.
    pub root: &'a Path,
    /// The `capabilities` object of the client.
    pub capabilities: Value,
    /// Notification methods to keep for `wait_notification`; others are dropped.
    pub keep_notifications: &'a [&'a str],
    pub language_id: fn(&Path) -> &'static str,
}

/// `REFAC_LSP_TRACE=1` prints every message of every session to stderr, cut
/// to 400 characters (`full` keeps them whole), to see what a server was told
/// and answered.
fn trace(direction: &str, server: &str, body: &str) {
    let Some(mode) = std::env::var_os("REFAC_LSP_TRACE") else {
        return;
    };
    let limit = if mode == "full" { usize::MAX } else { 400 };
    let cut: String = body.chars().take(limit).collect();
    eprintln!("{direction} {server}: {cut}");
}

pub struct LspSession {
    name: String,
    child: Child,
    stdin: ChildStdin,
    incoming: UnboundedReceiver<Value>,
    keep_notifications: Vec<String>,
    inbox: Vec<Value>,
    next_id: u64,
    versions: HashMap<PathBuf, i64>,
    language_id: fn(&Path) -> &'static str,
}

impl LspSession {
    /// Spawn the server and run `initialize`. Returns the session and the
    /// `initialize` result so the caller can check the server's capabilities
    /// before calling [`LspSession::initialized`].
    pub async fn start(config: SessionConfig<'_>) -> Result<(Self, Value)> {
        let mut child = Command::new(config.executable)
            .args(&config.args)
            .current_dir(config.cwd)
            .kill_on_drop(true)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .with_context(|| {
                format!(
                    "Could not start the {} {}",
                    config.name,
                    config.executable.display()
                )
            })?;
        let stdin = child
            .stdin
            .take()
            .with_context(|| format!("{} stdin was not piped", config.name))?;
        let stdout = child
            .stdout
            .take()
            .with_context(|| format!("{} stdout was not piped", config.name))?;
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
            name: config.name.to_string(),
            child,
            stdin,
            incoming,
            keep_notifications: config
                .keep_notifications
                .iter()
                .map(|m| m.to_string())
                .collect(),
            inbox: Vec::new(),
            next_id: 1,
            versions: HashMap::new(),
            language_id: config.language_id,
        };
        let uri = Url::from_directory_path(config.root)
            .map_err(|_| anyhow::anyhow!("Invalid project path {}", config.root.display()))?
            .to_string();
        let init = session
            .request(
                "initialize",
                json!({
                    "processId": std::process::id(),
                    "rootUri": uri,
                    "workspaceFolders": [{ "uri": uri, "name": "project" }],
                    "capabilities": config.capabilities,
                }),
            )
            .await?;
        Ok((session, init))
    }

    pub async fn initialized(&mut self) -> Result<()> {
        self.notify("initialized", json!({})).await
    }

    pub fn pid(&self) -> Option<u32> {
        self.child.id()
    }

    async fn send(&mut self, message: Value) -> Result<()> {
        let body = serde_json::to_string(&message)?;
        trace(">>", &self.name, &body);
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
                format!("The {} closed its connection during {method}", self.name)
            })?;
            let Some(response) = self.route(message).await? else {
                continue;
            };
            if response.get("id") != Some(&json!(id)) {
                continue;
            }
            if let Some(error) = response.get("error") {
                return Err(RpcError {
                    code: error["code"].as_i64().unwrap_or(0),
                    message: error["message"]
                        .as_str()
                        .unwrap_or("unknown error")
                        .to_string(),
                }
                .into());
            }
            return Ok(response["result"].clone());
        }
    }

    /// Wait until a kept notification satisfies `wanted`, consuming it. Kept
    /// notifications that arrived earlier count. Times out loudly.
    pub async fn wait_notification(
        &mut self,
        wanted: impl Fn(&Value) -> bool,
        timeout: Duration,
    ) -> Result<Value> {
        let deadline = Instant::now() + timeout;
        loop {
            if let Some(position) = self.inbox.iter().position(&wanted) {
                return Ok(self.inbox.remove(position));
            }
            let left = deadline.saturating_duration_since(Instant::now());
            let message = tokio::time::timeout(left, self.incoming.recv())
                .await
                .map_err(|_| {
                    anyhow::anyhow!(
                        "The {} did not signal readiness within {} seconds",
                        self.name,
                        timeout.as_secs()
                    )
                })?
                .with_context(|| {
                    format!(
                        "The {} closed its connection before it was ready",
                        self.name
                    )
                })?;
            self.route(message).await?;
        }
    }

    /// Route every message that has already arrived, without waiting for more.
    /// Lets a caller see whether the server has said something since its last
    /// look at the kept notifications.
    pub async fn poll(&mut self) -> Result<()> {
        while let Ok(message) = self.incoming.try_recv() {
            self.route(message).await?;
        }
        Ok(())
    }

    /// Kept notifications seen so far, oldest first, for error diagnostics.
    pub fn notifications(&self) -> &[Value] {
        &self.inbox
    }

    /// Answer server requests, keep wanted notifications, and hand back
    /// responses to the caller.
    async fn route(&mut self, message: Value) -> Result<Option<Value>> {
        trace("<<", &self.name, &message.to_string());
        let Some(method) = message.get("method").and_then(Value::as_str) else {
            return Ok(Some(message));
        };
        if let Some(request_id) = message.get("id") {
            self.answer_server_request(request_id.clone(), &message)
                .await?;
        } else if self.keep_notifications.iter().any(|kept| kept == method) {
            self.inbox.push(message);
        }
        Ok(None)
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
                let language_id = (self.language_id)(path);
                self.notify(
                    "textDocument/didOpen",
                    json!({ "textDocument": { "uri": uri, "languageId": language_id, "version": 1, "text": text } }),
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

    /// Close every open document at or below `path`, for files that were
    /// moved or deleted. A document the session never opened is ignored.
    pub async fn close_under(&mut self, path: &Path) -> Result<()> {
        let open: Vec<PathBuf> = self
            .versions
            .keys()
            .filter(|opened| opened.starts_with(path))
            .cloned()
            .collect();
        for opened in open {
            self.versions.remove(&opened);
            let uri = Url::from_file_path(&opened)
                .map_err(|_| anyhow::anyhow!("Invalid document path {}", opened.display()))?
                .to_string();
            self.notify(
                "textDocument/didClose",
                json!({ "textDocument": { "uri": uri } }),
            )
            .await?;
        }
        Ok(())
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
