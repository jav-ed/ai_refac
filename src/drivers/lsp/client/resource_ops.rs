//! The file operations a workspace edit may carry (create, rename, delete),
//! applied with the options the protocol defines.

use super::changes::uri_to_path;
use anyhow::Result;
use lsp_types::{CreateFile, DeleteFile, RenameFile, ResourceOp};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub async fn apply_resource_op(operation: ResourceOp) -> Result<Option<(PathBuf, PathBuf)>> {
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
