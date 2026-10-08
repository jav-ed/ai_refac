//! The path that should name the moved module after the move, given how a
//! reference spells it today.

use anyhow::{Context, Result};

/// `full` is the spelling at the reference (inherited group prefix included).
/// `None` means the reference does not name the moved module and stays as it is.
pub fn desired_reference_path(
    full: &[String],
    source: &[String],
    target: &[String],
    same_crate: bool,
) -> Result<Option<Vec<String>>> {
    let source_start = full.len().checked_sub(source.len());
    let encodes_source = source_start.is_some_and(|start| full[start..] == *source);
    if !same_crate && !encodes_source {
        return Ok(None);
    }
    if !same_crate && source_start == Some(0) {
        return Ok(None);
    }

    if same_crate {
        let mut canonical = vec!["crate".to_string()];
        canonical.extend_from_slice(target);
        return Ok(Some(canonical));
    }

    let start =
        source_start.context("External reference does not encode the source module path")?;
    let mut rewritten = full[..start].to_vec();
    rewritten.extend_from_slice(target);
    Ok(Some(rewritten))
}
