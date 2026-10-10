//! What refac announces to the Kotlin server and what it requires back.

use anyhow::{Result, bail};
use serde_json::{Value, json};

pub(super) fn client_capabilities() -> Value {
    json!({
        "workspace": {
            "applyEdit": true,
            "workspaceEdit": {
                "documentChanges": true,
                "resourceOperations": ["create", "rename", "delete"],
            },
            "workspaceFolders": true,
            "configuration": true,
            "fileOperations": { "willRename": true },
        },
        "window": { "workDoneProgress": true },
        "textDocument": {
            "rename": { "prepareSupport": true },
            "documentSymbol": { "hierarchicalDocumentSymbolSupport": true },
        },
    })
}

pub(super) fn check_capabilities(capabilities: &Value) -> Result<()> {
    if capabilities["workspace"]["fileOperations"]["willRename"].is_null() {
        bail!("The Kotlin language server does not offer workspace/willRenameFiles");
    }
    if capabilities["renameProvider"]["prepareProvider"] != true {
        bail!("The Kotlin language server does not offer prepareRename");
    }
    if capabilities["referencesProvider"].is_null()
        || capabilities["documentSymbolProvider"].is_null()
    {
        bail!("The Kotlin language server does not offer references and document symbols");
    }
    Ok(())
}
