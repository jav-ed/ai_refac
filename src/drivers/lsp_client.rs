use anyhow::{Context, Result};
use lsp_types::{
    AnnotatedTextEdit, ClientCapabilities, CreateFile, DeleteFile, DidOpenTextDocumentParams,
    DocumentChangeOperation, DocumentChanges, FailureHandlingKind, FileRename, InitializeParams,
    OneOf, Position, RenameFile, RenameFilesParams, ResourceOp, ResourceOperationKind,
    TextDocumentEdit, TextDocumentItem, TextEdit, Uri, WorkspaceClientCapabilities, WorkspaceEdit,
    WorkspaceEditClientCapabilities, WorkspaceFileOperationsClientCapabilities,
};
use serde_json::Value;
use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::{Path, PathBuf};
use std::process::Stdio;
use std::str::FromStr;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::{Child, ChildStdin, ChildStdout, Command};
use url::Url;

pub struct LspClient {
    binary_path: String,
}

/// A single symbol rename to be processed in a batch LSP session.
pub struct SymbolRenameRequest {
    pub document_path: std::path::PathBuf,
    pub position: lsp_types::Position,
    pub new_name: String,
    pub pending_moves: std::collections::HashMap<std::path::PathBuf, std::path::PathBuf>,
}

struct LspSession {
    child: Child,
    stdin: ChildStdin,
    reader: BufReader<ChildStdout>,
    root_dir: PathBuf,
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
        language_id: Option<&str>,
        file_extensions: &[&str],
    ) -> Result<()> {
        let mut session = self.start_session(args, root_path, true).await?;
        let result = async {
            if let Some(language_id) = language_id {
                Self::prime_workspace_documents(
                    &mut session.stdin,
                    &session.root_dir,
                    language_id,
                    file_extensions,
                )
                .await?;
            }

            tokio::time::sleep(Duration::from_millis(1_500)).await;

            let mut file_renames = Vec::new();
            for (source, target) in file_map {
                let source_abs = resolve_abs_path(&session.root_dir, Path::new(&source));
                let target_abs = resolve_abs_path(&session.root_dir, Path::new(&target));

                let source_uri = Url::from_file_path(&source_abs)
                    .map_err(|_| anyhow::anyhow!("Invalid source path"))?;
                let target_uri = Url::from_file_path(&target_abs)
                    .map_err(|_| anyhow::anyhow!("Invalid target path"))?;

                file_renames.push(FileRename {
                    old_uri: source_uri.to_string(),
                    new_uri: target_uri.to_string(),
                });
            }

            // Build a reverse map: new_abs_path → old_abs_path.
            // When the LSP returns a workspace edit targeting a file at its
            // *new* (not-yet-existing) path, we redirect the edit to the
            // source path so it is applied before the filesystem move.
            let mut pending_moves: HashMap<PathBuf, PathBuf> = HashMap::new();
            for rename in &file_renames {
                let old_url = Url::parse(&rename.old_uri).with_context(|| {
                    format!("Invalid old URI in rename: {}", rename.old_uri)
                })?;
                let new_url = Url::parse(&rename.new_uri).with_context(|| {
                    format!("Invalid new URI in rename: {}", rename.new_uri)
                })?;
                let old_path = old_url.to_file_path().map_err(|_| {
                    anyhow::anyhow!("Cannot convert old URI to file path: {}", rename.old_uri)
                })?;
                let new_path = new_url.to_file_path().map_err(|_| {
                    anyhow::anyhow!("Cannot convert new URI to file path: {}", rename.new_uri)
                })?;
                pending_moves.insert(new_path, old_path);
            }

            let rename_params = RenameFilesParams {
                files: file_renames,
            };

            Self::send_request(
                &mut session.stdin,
                "workspace/willRenameFiles",
                3,
                rename_params,
            )
            .await?;
            let resp = Self::read_response(&mut session.reader, 3).await?;
            tracing::debug!("LSP willRenameFiles response: {:?}", resp);

            if let Some(edit) = workspace_edit_from_response(&resp, "workspace/willRenameFiles")? {
                let _ = apply_workspace_edit(edit, &pending_moves).await?;
            }

            Ok(())
        }
        .await;

        Self::shutdown_session(&mut session).await;
        result
    }

    pub async fn initialize_and_rename_symbol(
        &self,
        args: &[&str],
        root_path: Option<&Path>,
        document_path: &Path,
        position: Position,
        new_name: &str,
        pending_moves: &HashMap<PathBuf, PathBuf>,
    ) -> Result<Vec<(PathBuf, PathBuf)>> {
        let mut session = self.start_session(args, root_path, false).await?;
        let result = async {
            tokio::time::sleep(Duration::from_millis(750)).await;

            let document_abs = resolve_abs_path(&session.root_dir, document_path);
            let document_uri = Url::from_file_path(&document_abs)
                .map_err(|_| anyhow::anyhow!("Invalid document path"))?;

            for attempt in 0..5 {
                let request_id = 10 + attempt;
                let rename_params = serde_json::json!({
                    "textDocument": { "uri": document_uri.as_str() },
                    "position": position,
                    "newName": new_name,
                });

                Self::send_request(
                    &mut session.stdin,
                    "textDocument/rename",
                    request_id,
                    rename_params,
                )
                .await?;
                let resp = Self::read_response(&mut session.reader, request_id).await?;

                if is_content_modified_response(&resp) && attempt < 4 {
                    tokio::time::sleep(Duration::from_millis(1_000)).await;
                    continue;
                }

                if let Some(edit) = workspace_edit_from_response(&resp, "textDocument/rename")? {
                    let (file_renames, _) = apply_workspace_edit(edit, pending_moves).await?;
                    return Ok(file_renames);
                }

                anyhow::bail!("textDocument/rename returned no workspace edit");
            }

            anyhow::bail!(
                "textDocument/rename did not succeed after retrying content-modified responses"
            )
        }
        .await;

        Self::shutdown_session(&mut session).await;
        result
    }

    pub async fn initialize_and_rename_symbols_batch(
        &self,
        args: &[&str],
        root_path: Option<&Path>,
        renames: Vec<SymbolRenameRequest>,
        language_id: &str,
    ) -> Result<Vec<Vec<(PathBuf, PathBuf)>>> {
        if renames.is_empty() {
            return Ok(Vec::new());
        }

        let mut session = self.start_session(args, root_path, false).await?;
        let language_id = language_id.to_string();
        let result = async {
            tokio::time::sleep(Duration::from_millis(750)).await;

            // Open all unique document paths so the LSP has them in its view
            // before we start sending rename requests.
            let unique_docs: std::collections::HashSet<PathBuf> = renames
                .iter()
                .map(|r| resolve_abs_path(&session.root_dir, &r.document_path))
                .collect();
            let mut opened_versions: HashMap<PathBuf, u32> = HashMap::new();

            for doc_path in &unique_docs {
                let Ok(url) = Url::from_file_path(doc_path) else {
                    continue;
                };
                let Ok(uri) = Uri::from_str(url.as_str()) else {
                    continue;
                };
                if let Ok(text) = tokio::fs::read_to_string(doc_path).await {
                    let params = DidOpenTextDocumentParams {
                        text_document: TextDocumentItem::new(
                            uri,
                            language_id.clone(),
                            1,
                            text,
                        ),
                    };
                    Self::send_notification(&mut session.stdin, "textDocument/didOpen", params)
                        .await?;
                    opened_versions.insert(doc_path.clone(), 1);
                }
            }

            let mut all_file_renames: Vec<Vec<(PathBuf, PathBuf)>> = Vec::new();

            for (idx, rename) in renames.into_iter().enumerate() {
                let document_abs =
                    resolve_abs_path(&session.root_dir, &rename.document_path);
                let document_uri = Url::from_file_path(&document_abs)
                    .map_err(|_| anyhow::anyhow!("Invalid document path"))?;

                let mut file_renames = Vec::new();

                for attempt in 0..5u64 {
                    let request_id = 10 + (idx as u64) * 10 + attempt;
                    let rename_params = serde_json::json!({
                        "textDocument": { "uri": document_uri.as_str() },
                        "position": rename.position,
                        "newName": rename.new_name,
                    });

                    Self::send_request(
                        &mut session.stdin,
                        "textDocument/rename",
                        request_id,
                        rename_params,
                    )
                    .await?;
                    let resp =
                        Self::read_response(&mut session.reader, request_id).await?;

                    if is_content_modified_response(&resp) && attempt < 4 {
                        tokio::time::sleep(Duration::from_millis(1_000)).await;
                        continue;
                    }

                    if let Some(edit) =
                        workspace_edit_from_response(&resp, "textDocument/rename")?
                    {
                        let (renames, modified_paths) =
                            apply_workspace_edit(edit, &rename.pending_moves).await?;
                        file_renames = renames;

                        // Notify the LSP of every file we just changed on disk so
                        // subsequent renames in this session see the updated state.
                        for path in modified_paths {
                            if let Ok(content) = tokio::fs::read_to_string(&path).await {
                                let version =
                                    opened_versions.get(&path).copied().unwrap_or(1) + 1;
                                opened_versions.insert(path.clone(), version);
                                let _ = Self::send_did_change(
                                    &mut session.stdin,
                                    &path,
                                    &content,
                                    version,
                                )
                                .await;
                            }
                        }

                        break;
                    }

                    anyhow::bail!("textDocument/rename returned no workspace edit");
                }

                all_file_renames.push(file_renames);
            }

            Ok(all_file_renames)
        }
        .await;

        Self::shutdown_session(&mut session).await;
        result
    }

    async fn send_did_change(
        stdin: &mut tokio::process::ChildStdin,
        path: &Path,
        content: &str,
        version: u32,
    ) -> Result<()> {
        let url = Url::from_file_path(path)
            .map_err(|_| anyhow::anyhow!("Invalid path for didChange: {:?}", path))?;
        let params = serde_json::json!({
            "textDocument": {
                "uri": url.as_str(),
                "version": version
            },
            "contentChanges": [{"text": content}]
        });
        Self::send_notification(stdin, "textDocument/didChange", params).await
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

    async fn send_notification<T: serde::Serialize>(
        stdin: &mut tokio::process::ChildStdin,
        method: &str,
        params: T,
    ) -> Result<()> {
        let params_json = serde_json::to_value(params)?;
        let notification = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params_json,
        });
        let body = serde_json::to_string(&notification)?;
        let message = format!("Content-Length: {}\r\n\r\n{}", body.len(), body);
        stdin.write_all(message.as_bytes()).await?;
        stdin.flush().await?;
        Ok(())
    }

    async fn prime_workspace_documents(
        stdin: &mut tokio::process::ChildStdin,
        root_dir: &Path,
        language_id: &str,
        file_extensions: &[&str],
    ) -> Result<()> {
        if file_extensions.is_empty() {
            return Ok(());
        }

        for path in collect_workspace_documents(root_dir, file_extensions)? {
            let uri = Uri::from_str(
                Url::from_file_path(&path)
                    .map_err(|_| anyhow::anyhow!("Invalid workspace document path: {:?}", path))?
                    .as_str(),
            )
            .map_err(|error| anyhow::anyhow!("Invalid workspace document URI: {}", error))?;

            let text = tokio::fs::read_to_string(&path).await?;
            let params = DidOpenTextDocumentParams {
                text_document: TextDocumentItem::new(uri, language_id.to_string(), 1, text),
            };

            Self::send_notification(stdin, "textDocument/didOpen", params).await?;
        }

        Ok(())
    }

    async fn read_response(
        reader: &mut BufReader<tokio::process::ChildStdout>,
        expected_id: u64,
    ) -> Result<Value> {
        tokio::time::timeout(
            Duration::from_secs(60),
            Self::read_response_inner(reader, expected_id),
        )
        .await
        .map_err(|_| anyhow::anyhow!("LSP timed out waiting for response (id={})", expected_id))?
    }

    async fn read_response_inner(
        reader: &mut BufReader<tokio::process::ChildStdout>,
        expected_id: u64,
    ) -> Result<Value> {
        let expected_id = Value::from(expected_id);

        loop {
            let response = Self::read_message(reader).await?;

            if response.get("id") == Some(&expected_id) {
                return Ok(response);
            }

            if let Some(method) = response.get("method").and_then(Value::as_str) {
                tracing::debug!(
                    "Ignoring unrelated LSP message while waiting for response: {method}"
                );
            } else {
                tracing::debug!(
                    "Ignoring unrelated LSP payload while waiting for response: {:?}",
                    response
                );
            }
        }
    }

    async fn read_message(reader: &mut BufReader<tokio::process::ChildStdout>) -> Result<Value> {
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

    async fn start_session(
        &self,
        args: &[&str],
        root_path: Option<&Path>,
        advertise_file_operations: bool,
    ) -> Result<LspSession> {
        let mut child = Command::new(&self.binary_path)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        let mut stdin = child.stdin.take().context("Failed to open stdin")?;
        let stdout = child.stdout.take().context("Failed to open stdout")?;
        let mut reader = BufReader::new(stdout);

        let root_dir = if let Some(root) = root_path {
            root.to_path_buf()
        } else {
            std::env::current_dir()?
        };

        let root_uri = Url::from_directory_path(&root_dir)
            .map_err(|_| anyhow::anyhow!("Invalid root path"))?;

        #[allow(deprecated)]
        let init_params = InitializeParams {
            process_id: Some(std::process::id()),
            root_uri: None,
            workspace_folders: Some(vec![lsp_types::WorkspaceFolder {
                uri: Uri::from_str(root_uri.as_str())
                    .map_err(|e| anyhow::anyhow!("Invalid URI: {}", e))?,
                name: "project".to_string(),
            }]),
            capabilities: ClientCapabilities {
                workspace: Some(WorkspaceClientCapabilities {
                    apply_edit: Some(true),
                    workspace_edit: Some(WorkspaceEditClientCapabilities {
                        document_changes: Some(true),
                        resource_operations: Some(vec![
                            ResourceOperationKind::Create,
                            ResourceOperationKind::Rename,
                            ResourceOperationKind::Delete,
                        ]),
                        failure_handling: Some(FailureHandlingKind::Transactional),
                        normalizes_line_endings: Some(false),
                        change_annotation_support: None,
                    }),
                    workspace_folders: Some(true),
                    file_operations: advertise_file_operations.then_some(
                        WorkspaceFileOperationsClientCapabilities {
                            will_rename: Some(true),
                            ..Default::default()
                        },
                    ),
                    ..Default::default()
                }),
                ..Default::default()
            },
            ..Default::default()
        };

        Self::send_request(&mut stdin, "initialize", 1, init_params).await?;
        let init_resp = Self::read_response(&mut reader, 1).await?;
        ensure_response_ok(&init_resp, "initialize")?;

        Self::send_notification(&mut stdin, "initialized", serde_json::json!({})).await?;

        Ok(LspSession {
            child,
            stdin,
            reader,
            root_dir,
        })
    }

    async fn shutdown_session(session: &mut LspSession) {
        let _ = Self::send_request(&mut session.stdin, "shutdown", 99, ()).await;
        let _ = session.child.kill().await;
    }
}

fn ensure_response_ok(response: &Value, method: &str) -> Result<()> {
    if let Some(error) = response.get("error") {
        let code = error
            .get("code")
            .and_then(Value::as_i64)
            .unwrap_or_default();
        let message = error
            .get("message")
            .and_then(Value::as_str)
            .unwrap_or("Unknown LSP error");
        anyhow::bail!("{method} failed ({code}): {message}");
    }

    Ok(())
}

fn workspace_edit_from_response(response: &Value, method: &str) -> Result<Option<WorkspaceEdit>> {
    ensure_response_ok(response, method)?;

    match response.get("result") {
        Some(Value::Null) | None => Ok(None),
        Some(result) => Ok(Some(serde_json::from_value(result.clone()).with_context(
            || format!("Failed to parse {method} response as WorkspaceEdit"),
        )?)),
    }
}

fn is_content_modified_response(response: &Value) -> bool {
    response
        .get("error")
        .and_then(|error| {
            let code = error.get("code").and_then(Value::as_i64);
            let message = error.get("message").and_then(Value::as_str);
            Some(
                code == Some(-32801)
                    || message
                        .map(|message| message.eq_ignore_ascii_case("content modified"))
                        .unwrap_or(false),
            )
        })
        .unwrap_or(false)
}

fn resolve_abs_path(root_dir: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root_dir.join(path)
    }
}

async fn apply_workspace_edit(
    edit: WorkspaceEdit,
    pending_moves: &HashMap<PathBuf, PathBuf>,
) -> Result<(Vec<(PathBuf, PathBuf)>, Vec<PathBuf>)> {
    // Phase 1: Collect all changes without touching the filesystem.
    let changes = collect_pending_changes(edit, pending_moves)?;

    // Phase 2: Validate — every text-edit target must exist before we write
    // anything. Fail fast so no partial edits are applied.
    for change in &changes {
        if let PendingChange::TextEdit { path, .. } = change {
            if !path.exists() {
                anyhow::bail!(
                    "LSP returned a text edit for a file that does not exist: {:?}",
                    path
                );
            }
        }
    }

    // Phase 3: Execute all changes in their original order.
    let mut file_renames: Vec<(PathBuf, PathBuf)> = Vec::new();
    let mut text_edited_paths: Vec<PathBuf> = Vec::new();

    for change in changes {
        match change {
            PendingChange::TextEdit { path, edits } => {
                text_edited_paths.push(path.clone());
                apply_text_edits_to_path(&path, edits).await?;
            }
            PendingChange::ResourceOp(op) => {
                if let Some(rename) = apply_resource_op(op).await? {
                    file_renames.push(rename);
                }
            }
        }
    }

    Ok((file_renames, text_edited_paths))
}

enum PendingChange {
    TextEdit { path: PathBuf, edits: Vec<TextEdit> },
    ResourceOp(ResourceOp),
}

fn collect_pending_changes(
    edit: WorkspaceEdit,
    pending_moves: &HashMap<PathBuf, PathBuf>,
) -> Result<Vec<PendingChange>> {
    let mut changes = Vec::new();

    if let Some(document_changes) = edit.document_changes {
        match document_changes {
            DocumentChanges::Edits(edits) => {
                for edit in edits {
                    collect_text_document_edit(edit, pending_moves, &mut changes)?;
                }
            }
            DocumentChanges::Operations(ops) => {
                for op in ops {
                    match op {
                        DocumentChangeOperation::Edit(edit) => {
                            collect_text_document_edit(edit, pending_moves, &mut changes)?;
                        }
                        DocumentChangeOperation::Op(resource_op) => {
                            changes.push(PendingChange::ResourceOp(resource_op));
                        }
                    }
                }
            }
        }
    } else if let Some(text_changes) = edit.changes {
        for (uri, edits) in text_changes {
            let path = uri_to_path(&uri)?;
            let effective = redirect_if_pending(&path, pending_moves);
            changes.push(PendingChange::TextEdit { path: effective, edits });
        }
    }

    Ok(changes)
}

fn collect_text_document_edit(
    edit: TextDocumentEdit,
    pending_moves: &HashMap<PathBuf, PathBuf>,
    out: &mut Vec<PendingChange>,
) -> Result<()> {
    let path = uri_to_path(&edit.text_document.uri)?;
    let effective = redirect_if_pending(&path, pending_moves);
    let edits = edit
        .edits
        .into_iter()
        .map(|e| match e {
            OneOf::Left(edit) => edit,
            OneOf::Right(AnnotatedTextEdit { text_edit, .. }) => text_edit,
        })
        .collect();
    out.push(PendingChange::TextEdit { path: effective, edits });
    Ok(())
}

fn redirect_if_pending(path: &Path, pending_moves: &HashMap<PathBuf, PathBuf>) -> PathBuf {
    if !path.exists() {
        if let Some(source) = pending_moves.get(path) {
            return source.clone();
        }
    }
    path.to_path_buf()
}

async fn apply_resource_op(operation: ResourceOp) -> Result<Option<(PathBuf, PathBuf)>> {
    match operation {
        ResourceOp::Create(op) => {
            apply_create_file(op).await?;
            Ok(None)
        }
        ResourceOp::Rename(op) => apply_rename_file(op).await,
        ResourceOp::Delete(op) => {
            apply_delete_file(op).await?;
            Ok(None)
        }
    }
}

async fn apply_create_file(operation: CreateFile) -> Result<()> {
    let path = uri_to_path(&operation.uri)?;
    let options = operation.options.as_ref();

    if let Some(parent) = path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    match tokio::fs::metadata(&path).await {
        Ok(metadata) => {
            if options
                .and_then(|options| options.overwrite)
                .unwrap_or(false)
            {
                remove_existing_path(&path, metadata.is_dir()).await?;
            } else if options
                .and_then(|options| options.ignore_if_exists)
                .unwrap_or(false)
            {
                return Ok(());
            } else {
                anyhow::bail!("CreateFile target already exists: {:?}", path);
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }

    tokio::fs::File::create(path).await?;
    Ok(())
}

async fn apply_rename_file(operation: RenameFile) -> Result<Option<(PathBuf, PathBuf)>> {
    let old_path = uri_to_path(&operation.old_uri)?;
    let new_path = uri_to_path(&operation.new_uri)?;
    let options = operation.options.as_ref();

    if let Some(parent) = new_path.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }

    match tokio::fs::metadata(&old_path).await {
        Ok(_) => {}
        Err(error) if error.kind() == ErrorKind::NotFound => {
            if new_path.exists() {
                return Ok(Some((old_path, new_path)));
            }

            if options
                .and_then(|options| options.ignore_if_exists)
                .unwrap_or(false)
            {
                return Ok(None);
            }

            return Err(error.into());
        }
        Err(error) => return Err(error.into()),
    }

    match tokio::fs::metadata(&new_path).await {
        Ok(metadata) => {
            if options
                .and_then(|options| options.overwrite)
                .unwrap_or(false)
            {
                remove_existing_path(&new_path, metadata.is_dir()).await?;
            } else if options
                .and_then(|options| options.ignore_if_exists)
                .unwrap_or(false)
            {
                return Ok(None);
            } else {
                anyhow::bail!("RenameFile target already exists: {:?}", new_path);
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }

    tokio::fs::rename(&old_path, &new_path).await?;
    Ok(Some((old_path, new_path)))
}

async fn apply_delete_file(operation: DeleteFile) -> Result<()> {
    let path = uri_to_path(&operation.uri)?;
    let options = operation.options.as_ref();

    match tokio::fs::metadata(&path).await {
        Ok(metadata) => {
            if metadata.is_dir() {
                if options
                    .and_then(|options| options.recursive)
                    .unwrap_or(false)
                {
                    tokio::fs::remove_dir_all(path).await?;
                } else {
                    tokio::fs::remove_dir(path).await?;
                }
            } else {
                tokio::fs::remove_file(path).await?;
            }
        }
        Err(error) if error.kind() == ErrorKind::NotFound => {
            if options
                .and_then(|options| options.ignore_if_not_exists)
                .unwrap_or(false)
            {
                return Ok(());
            }

            return Err(error.into());
        }
        Err(error) => return Err(error.into()),
    }

    Ok(())
}

async fn remove_existing_path(path: &Path, is_dir: bool) -> Result<()> {
    if is_dir {
        tokio::fs::remove_dir_all(path).await?;
    } else {
        tokio::fs::remove_file(path).await?;
    }

    Ok(())
}

async fn apply_text_edits_to_path(path: &Path, edits: Vec<TextEdit>) -> Result<()> {
    let original = tokio::fs::read_to_string(path).await?;
    let updated = apply_text_edits(&original, edits)?;

    if updated != original {
        tokio::fs::write(path, updated).await?;
    }

    Ok(())
}

fn apply_text_edits(content: &str, edits: Vec<TextEdit>) -> Result<String> {
    let line_offsets = line_start_offsets(content);
    let mut edits_with_offsets = Vec::with_capacity(edits.len());

    for edit in edits {
        let start = position_to_byte_offset(content, &line_offsets, edit.range.start)?;
        let end = position_to_byte_offset(content, &line_offsets, edit.range.end)?;

        if start > end {
            anyhow::bail!(
                "Invalid edit range: start {:?} is after end {:?}",
                start,
                end
            );
        }

        edits_with_offsets.push((start, end, edit.new_text));
    }

    edits_with_offsets.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| b.1.cmp(&a.1)));

    let mut updated = content.to_string();
    for (start, end, new_text) in edits_with_offsets {
        updated.replace_range(start..end, &new_text);
    }

    Ok(updated)
}

fn line_start_offsets(content: &str) -> Vec<usize> {
    let mut offsets = vec![0];
    for (index, byte) in content.bytes().enumerate() {
        if byte == b'\n' {
            offsets.push(index + 1);
        }
    }
    offsets
}

fn position_to_byte_offset(
    content: &str,
    line_offsets: &[usize],
    position: Position,
) -> Result<usize> {
    let line = position.line as usize;
    if line >= line_offsets.len() {
        anyhow::bail!(
            "LSP edit line {} is out of bounds for document with {} lines",
            line,
            line_offsets.len()
        );
    }

    let line_start = line_offsets[line];
    let next_line_start = line_offsets.get(line + 1).copied().unwrap_or(content.len());
    let line_end = trim_line_ending(content, line_start, next_line_start);
    let line_text = &content[line_start..line_end];
    let target_character = position.character as usize;

    if target_character == 0 {
        return Ok(line_start);
    }

    let mut utf16_offset = 0;
    for (byte_index, ch) in line_text.char_indices() {
        if utf16_offset >= target_character {
            return Ok(line_start + byte_index);
        }

        let next_utf16_offset = utf16_offset + ch.len_utf16();
        if next_utf16_offset > target_character {
            return Ok(line_start + byte_index);
        }

        utf16_offset = next_utf16_offset;
    }

    Ok(line_end)
}

fn trim_line_ending(content: &str, line_start: usize, next_line_start: usize) -> usize {
    let mut end = next_line_start;

    if end > line_start && content.as_bytes()[end - 1] == b'\n' {
        end -= 1;
    }

    if end > line_start && content.as_bytes()[end - 1] == b'\r' {
        end -= 1;
    }

    end
}

fn uri_to_path(uri: &Uri) -> Result<PathBuf> {
    let url = Url::parse(&uri.to_string()).map_err(|e| anyhow::anyhow!("Invalid URI: {}", e))?;
    url.to_file_path()
        .map_err(|_| anyhow::anyhow!("Cannot convert URI to file path"))
}

fn collect_workspace_documents(root_dir: &Path, file_extensions: &[&str]) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    collect_workspace_documents_recursive(root_dir, file_extensions, &mut files)?;
    Ok(files)
}

fn collect_workspace_documents_recursive(
    dir: &Path,
    file_extensions: &[&str],
    files: &mut Vec<PathBuf>,
) -> Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;

        if file_type.is_dir() {
            if should_skip_workspace_dir(&path) {
                continue;
            }

            collect_workspace_documents_recursive(&path, file_extensions, files)?;
            continue;
        }

        if file_type.is_file()
            && path
                .extension()
                .and_then(|ext| ext.to_str())
                .map(|ext| file_extensions.contains(&ext))
                .unwrap_or(false)
        {
            files.push(path);
        }
    }

    Ok(())
}

fn should_skip_workspace_dir(path: &Path) -> bool {
    matches!(
        path.file_name().and_then(|name| name.to_str()),
        Some(".git")
            | Some(".venv")
            | Some("__pycache__")
            | Some("node_modules")
            | Some("target")
            | Some(".dart_tool")
            | Some("build")
            | Some("dist")
            | Some("out")
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use lsp_types::{OptionalVersionedTextDocumentIdentifier, Range};
    use std::collections::HashMap;
    use tempfile::tempdir;

    #[test]
    fn test_apply_text_edits_rewrites_ascii_content() -> Result<()> {
        let content = "mod a;\nfn main() { a::value(); }\n";
        let edits = vec![
            TextEdit {
                range: Range {
                    start: Position::new(0, 4),
                    end: Position::new(0, 5),
                },
                new_text: "b".to_string(),
            },
            TextEdit {
                range: Range {
                    start: Position::new(1, 12),
                    end: Position::new(1, 13),
                },
                new_text: "b".to_string(),
            },
        ];

        let updated = apply_text_edits(content, edits)?;
        assert_eq!(updated, "mod b;\nfn main() { b::value(); }\n");
        Ok(())
    }

    #[test]
    fn test_apply_text_edits_handles_utf16_positions() -> Result<()> {
        let content = "🙂value\n";
        let edits = vec![TextEdit {
            range: Range {
                start: Position::new(0, 2),
                end: Position::new(0, 7),
            },
            new_text: "name".to_string(),
        }];

        let updated = apply_text_edits(content, edits)?;
        assert_eq!(updated, "🙂name\n");
        Ok(())
    }

    #[tokio::test]
    async fn test_apply_workspace_edit_writes_changes_map() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("main.rs");
        tokio::fs::write(&file_path, "mod a;\n").await?;

        let uri = Uri::from_str(Url::from_file_path(&file_path).unwrap().as_str()).unwrap();

        let mut changes = HashMap::new();
        changes.insert(
            uri,
            vec![TextEdit {
                range: Range {
                    start: Position::new(0, 4),
                    end: Position::new(0, 5),
                },
                new_text: "b".to_string(),
            }],
        );

        let _ = apply_workspace_edit(
            WorkspaceEdit {
                changes: Some(changes),
                document_changes: None,
                change_annotations: None,
            },
            &HashMap::new(),
        )
        .await?;

        assert_eq!(tokio::fs::read_to_string(file_path).await?, "mod b;\n");
        Ok(())
    }

    #[tokio::test]
    async fn test_apply_workspace_edit_writes_document_changes() -> Result<()> {
        let dir = tempdir()?;
        let file_path = dir.path().join("main.rs");
        tokio::fs::write(&file_path, "mod a;\n").await?;

        let uri = Uri::from_str(Url::from_file_path(&file_path).unwrap().as_str()).unwrap();

        let edit = TextDocumentEdit {
            text_document: OptionalVersionedTextDocumentIdentifier { uri, version: None },
            edits: vec![OneOf::Left(TextEdit {
                range: Range {
                    start: Position::new(0, 4),
                    end: Position::new(0, 5),
                },
                new_text: "b".to_string(),
            })],
        };

        let _ = apply_workspace_edit(
            WorkspaceEdit {
                changes: None,
                document_changes: Some(DocumentChanges::Edits(vec![edit])),
                change_annotations: None,
            },
            &HashMap::new(),
        )
        .await?;

        assert_eq!(tokio::fs::read_to_string(file_path).await?, "mod b;\n");
        Ok(())
    }
}
