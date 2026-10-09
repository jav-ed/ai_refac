//! `move --dry-run`: every group is planned by its driver and nothing is
//! written. The request goes through the same checks as a real move (grouping,
//! the TypeScript size limit, every tool present), a group the driver refuses
//! fails the dry run as it would fail the move, and the report has the shape of
//! the real one: per language the paths that move, then the edits per file.

use super::RefactorRequest;
use super::report::{FailedGroup, Pairs, capitalize, render_failed};
use crate::drivers::MovePreview;
use crate::logic::grouping::prepare::{Prepared, prepare};
use anyhow::{Result, bail};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

/// What a dry run found, as text and as data for `--json`. Paths are relative
/// to the project path where they lie below it.
#[derive(Debug)]
pub struct DryRun {
    pub text: String,
    pub moves: Vec<(String, String)>,
    pub files: Vec<(String, usize)>,
    pub notes: Vec<String>,
}

/// Plans a move and changes nothing. A request that could not be planned in
/// full is an error with the whole report as its text, like a real move.
pub async fn plan_refactor(req: RefactorRequest) -> Result<DryRun> {
    // Relative paths in the plan are made from one absolute project path.
    let req = super::absolute_project(req)?;
    let Prepared {
        root,
        groups,
        skipped,
        typescript_source_count: _,
    } = prepare(&req).await?;

    let mut planned: BTreeMap<String, MovePreview> = BTreeMap::new();
    let mut failed: Vec<FailedGroup> = Vec::new();
    let mut markdown_pairs: Pairs = Vec::new();
    let mut other_pairs: Pairs = Vec::new();
    let single_kotlin = super::kotlin_cost::is_single_move(
        groups
            .iter()
            .map(|(lang, files, _)| (lang.as_str(), files.as_slice())),
        root,
    );
    for (lang, files, driver) in groups {
        if lang == "markdown" {
            markdown_pairs = files;
            continue;
        }
        let started = std::time::Instant::now();
        match driver.plan_move(files.clone(), root).await {
            Ok(mut preview) => {
                if single_kotlin && lang == "kotlin" {
                    preview
                        .notes
                        .push(super::kotlin_cost::note("move", started.elapsed(), true));
                }
                other_pairs.extend(files);
                planned.insert(lang, preview);
            }
            Err(error) => failed.push(FailedGroup {
                lang,
                files,
                error: format!("{error:#}"),
            }),
        }
    }

    // Markdown is planned as a whole: its own files and the links that follow
    // everything the other languages move.
    if !markdown_pairs.is_empty() || !other_pairs.is_empty() {
        match crate::drivers::markdown::plan_move_with_links(
            markdown_pairs.clone(),
            other_pairs,
            root,
        )
        .await
        {
            Ok(preview) => {
                planned.insert("markdown".to_string(), preview);
            }
            Err(error) => failed.push(FailedGroup {
                lang: "markdown".to_string(),
                files: markdown_pairs,
                error: format!("{error:#}"),
            }),
        }
    }

    let base = Base::new(root);
    let text = render(&planned, &failed, &skipped, &base);
    if !failed.is_empty() || planned.is_empty() {
        bail!("{text}");
    }
    Ok(data(planned, text, &base))
}

/// The paths of the plan as they are shown: below the project path they are
/// relative to it.
struct Base {
    roots: Vec<PathBuf>,
}

impl Base {
    fn new(root: Option<&Path>) -> Self {
        let root = root
            .map(Path::to_path_buf)
            .or_else(|| std::env::current_dir().ok());
        let mut roots = Vec::new();
        if let Some(root) = root {
            roots.extend(root.canonicalize().ok());
            roots.push(root);
        }
        Self { roots }
    }

    fn show(&self, path: &Path) -> String {
        self.roots
            .iter()
            .find_map(|root| path.strip_prefix(root).ok())
            .unwrap_or(path)
            .display()
            .to_string()
    }
}

fn data(planned: BTreeMap<String, MovePreview>, text: String, base: &Base) -> DryRun {
    let mut moves = Vec::new();
    let mut files: BTreeMap<String, usize> = BTreeMap::new();
    let mut notes = Vec::new();
    for (lang, preview) in planned {
        moves.extend(
            preview
                .moves
                .iter()
                .map(|(from, to)| (base.show(from), base.show(to))),
        );
        for (path, count) in &preview.edits {
            *files.entry(base.show(path)).or_insert(0) += count;
        }
        notes.extend(
            preview
                .notes
                .iter()
                .map(|note| format!("{}: {note}", capitalize(&lang))),
        );
    }
    DryRun {
        text,
        moves,
        files: files.into_iter().collect(),
        notes,
    }
}

fn plural(count: usize, word: &str) -> String {
    format!("{count} {word}{}", if count == 1 { "" } else { "s" })
}

fn render(
    planned: &BTreeMap<String, MovePreview>,
    failed: &[FailedGroup],
    skipped: &[String],
    base: &Base,
) -> String {
    let moved: usize = planned.values().map(|preview| preview.moves.len()).sum();
    let edits: usize = planned.values().map(MovePreview::total_edits).sum();
    let mut text = match (planned.is_empty(), failed.is_empty()) {
        (true, true) => "// Dry run: nothing was changed. Nothing would be moved.\n".to_string(),
        (_, true) => format!(
            "// Dry run: nothing was changed. {} would move, with {} to update.\n",
            plural(moved, "path"),
            plural(edits, "edit")
        ),
        (_, false) => format!(
            "// Dry run: nothing was changed. The request cannot be carried out: {} could not be planned (the groups that could are listed).\n",
            plural(
                failed.iter().map(|group| group.files.len()).sum(),
                "requested path"
            )
        ),
    };
    for (lang, preview) in planned {
        text.push_str(&format!("\n// {} results:\n\n", capitalize(lang)));
        for (from, to) in &preview.moves {
            text.push_str(&format!("{} -> {}  \n", base.show(from), base.show(to)));
        }
        for (path, count) in &preview.edits {
            text.push_str(&format!(
                "// {} ({})  \n",
                base.show(path),
                plural(*count, "edit")
            ));
        }
        for note in &preview.notes {
            text.push_str(&format!("\n// Note: {note}  \n"));
        }
    }
    if !failed.is_empty() {
        render_failed(&mut text, failed, |path| path.to_string());
    }
    if !skipped.is_empty() {
        text.push_str("\n// Skipped (unsupported extension):  \n\n");
        for file in skipped {
            text.push_str(&format!("{file}  \n"));
        }
    }
    if failed.is_empty() && !planned.is_empty() {
        text.push_str("\n// Run the same command without --dry-run to apply the move.\n");
    }
    text
}
