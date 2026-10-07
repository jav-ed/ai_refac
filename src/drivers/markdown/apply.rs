//! Carrying out a plan: move what has to move, then write the changed
//! Markdown files. Everything is undone if any step fails, so a failed move
//! leaves the project as it was.

use super::plan::{LinkPlan, When};
use anyhow::{Context, Result, bail};

pub(crate) async fn apply(plan: &LinkPlan) -> Result<()> {
    let mut moved = Vec::new();
    let mut touched = Vec::new();
    let Err(error) = run(plan, &mut moved, &mut touched).await else {
        return Ok(());
    };

    let problems = undo(plan, &moved, &touched).await;
    if problems.is_empty() {
        return Err(error.context("The move failed and was rolled back; nothing was changed"));
    }
    Err(error.context(format!(
        "The move failed and the rollback was incomplete: {}",
        problems.join("; ")
    )))
}

async fn run(plan: &LinkPlan, moved: &mut Vec<usize>, touched: &mut Vec<usize>) -> Result<()> {
    if plan.when == When::BeforeMoving {
        for (index, entry) in plan.moves.moves().iter().enumerate() {
            if entry.to.exists() {
                bail!("Target already exists: {}", entry.to.display());
            }
            if let Some(parent) = entry.to.parent() {
                tokio::fs::create_dir_all(parent)
                    .await
                    .with_context(|| format!("Cannot create {}", parent.display()))?;
            }
            tokio::fs::rename(&entry.from, &entry.to)
                .await
                .with_context(|| {
                    format!(
                        "Cannot move {} to {}",
                        entry.from.display(),
                        entry.to.display()
                    )
                })?;
            moved.push(index);
        }
    }

    for (index, write) in plan.writes.iter().enumerate() {
        // Counted before the write, so a write that fails halfway is restored too.
        touched.push(index);
        tokio::fs::write(&write.destination, &write.after)
            .await
            .with_context(|| {
                format!(
                    "Failed to write updated Markdown to {}",
                    write.destination.display()
                )
            })?;
    }
    Ok(())
}

async fn undo(plan: &LinkPlan, moved: &[usize], touched: &[usize]) -> Vec<String> {
    let mut problems = Vec::new();
    for &index in touched.iter().rev() {
        let write = &plan.writes[index];
        // A write that failed before it created the file has nothing to undo.
        if !write.destination.exists() {
            continue;
        }
        if let Err(error) = tokio::fs::write(&write.destination, &write.before).await {
            problems.push(format!(
                "restoring {}: {error}",
                write.destination.display()
            ));
        }
    }
    for &index in moved.iter().rev() {
        let entry = &plan.moves.moves()[index];
        if let Err(error) = tokio::fs::rename(&entry.to, &entry.from).await {
            problems.push(format!("moving {} back: {error}", entry.to.display()));
        }
    }
    problems
}

#[cfg(test)]
mod tests;
