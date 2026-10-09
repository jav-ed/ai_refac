use super::plan::Plan;
use anyhow::{Result, bail};
use std::path::Path;

/// Write the verified plan. Every file is re-read first: a file edited since
/// planning aborts the rename before the first write. A failed write restores
/// the files already written.
pub fn apply(plan: &Plan) -> Result<()> {
    for file in &plan.files {
        if std::fs::read_to_string(&file.path)? != with_bom(file.bom, &file.old) {
            bail!(
                "Source changed while planning; refusing to overwrite {}",
                file.path.display()
            );
        }
    }
    let mut written: Vec<&Path> = Vec::new();
    for file in &plan.files {
        if let Err(error) = std::fs::write(&file.path, with_bom(file.bom, &file.new)) {
            let mut failures = Vec::new();
            for path in written.iter().rev() {
                let original = plan.files.iter().find(|candidate| candidate.path == *path);
                if let Some(original) = original
                    && let Err(failure) =
                        std::fs::write(path, with_bom(original.bom, &original.old))
                {
                    failures.push(format!("{}: {failure}", path.display()));
                }
            }
            if failures.is_empty() {
                bail!(
                    "Writing {} failed; original files restored: {error}",
                    file.path.display()
                );
            }
            bail!(
                "Writing {} failed ({error}) and rollback was incomplete; inspect the working tree:\n{}",
                file.path.display(),
                failures.join("\n")
            );
        }
        written.push(&file.path);
    }
    Ok(())
}

/// Take a written plan back, for a batch whose later rename failed. Every file
/// must still read as the plan wrote it: a file edited since is not
/// overwritten with the text from before the rename.
pub fn revert(plan: &Plan) -> Result<()> {
    for file in &plan.files {
        if std::fs::read_to_string(&file.path)? != with_bom(file.bom, &file.new) {
            bail!(
                "{} changed after the rename wrote it; it was not restored",
                file.path.display()
            );
        }
    }
    for file in &plan.files {
        std::fs::write(&file.path, with_bom(file.bom, &file.old))?;
    }
    Ok(())
}

pub fn with_bom(bom: bool, text: &str) -> String {
    if bom {
        format!("\u{FEFF}{text}")
    } else {
        text.to_string()
    }
}
