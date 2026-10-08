//! Symbol renames planned and not applied, for `move --dry-run`. A real batch
//! applies each rename before it asks for the next; here every request is
//! answered against the files as they are now, and `summarize` refuses plans
//! that edit the same text.

use super::{LspClient, PendingChange, SymbolRenameRequest, rename_edit, show_documents};
use crate::drivers::lsp::client::changes::collect_pending_changes;
use anyhow::Result;
use std::path::Path;

impl LspClient {
    /// The changes each rename would make, in request order. Nothing is written.
    pub async fn plan_symbol_renames(
        &self,
        args: &[&str],
        root_path: Option<&Path>,
        renames: Vec<SymbolRenameRequest>,
        language_id: &str,
    ) -> Result<Vec<Vec<PendingChange>>> {
        if renames.is_empty() {
            return Ok(Vec::new());
        }
        let mut started = self.start(args, root_path, language_id, false).await?;
        let outcome = async {
            show_documents(&mut started, &renames).await?;
            let mut plans = Vec::new();
            for rename in &renames {
                let edit = rename_edit(&mut started, rename).await?;
                plans.push(collect_pending_changes(edit, &rename.pending_moves)?);
            }
            Ok(plans)
        }
        .await;
        started.session.shutdown().await;
        outcome
    }
}
