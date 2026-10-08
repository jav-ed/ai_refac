//! The text a `move` answers with: what moved per language, the notes the
//! drivers add, the Markdown links that followed, and what failed or was
//! skipped. Lines starting with `//` are refac's own report.

use super::go_collaterals::detect_go_collaterals;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;

/// (source, target) as the user wrote them.
pub(super) type Pairs = Vec<(String, String)>;

/// One language's group of files that could not be moved. The group was put
/// back as a whole.
pub(super) struct FailedGroup {
    pub lang: String,
    pub files: Pairs,
    pub error: String,
}

/// Everything a finished `move` has to say.
pub(super) struct MoveOutcome<'a> {
    pub root: Option<&'a Path>,
    /// Moved files by language, in language order.
    pub moved: BTreeMap<String, Pairs>,
    /// What each driver reports beyond success, by language.
    pub notes: HashMap<String, Vec<String>>,
    /// Markdown links that were rewritten for the files other languages moved.
    pub link_notes: Option<Vec<String>>,
    pub failed: Vec<FailedGroup>,
    /// Sources whose extension no driver handles.
    pub skipped: Vec<String>,
    pub typescript_source_count: usize,
}

impl MoveOutcome<'_> {
    /// Whether the request was carried out as asked. A group that failed is
    /// not, and neither is a request in which nothing moved at all.
    pub fn is_complete(&self) -> bool {
        self.failed.is_empty() && !self.moved.is_empty()
    }

    pub fn render(&self) -> String {
        let mut response = self.headline();
        for (lang, files) in &self.moved {
            self.render_language(&mut response, lang, files);
        }
        if let Some(link_notes) = &self.link_notes {
            response.push_str("\n// Markdown links to the moved files:\n");
            for note in link_notes {
                response.push_str(&format!("\n// Note: {note}  \n"));
            }
        }
        if !self.failed.is_empty() {
            self.render_failed(&mut response);
        }
        if !self.skipped.is_empty() {
            response.push_str("\n// Skipped (unsupported extension):  \n\n");
            for file in &self.skipped {
                response.push_str(&format!("{}  \n", self.show(file)));
            }
        }
        response
    }

    fn headline(&self) -> String {
        let total: usize = self.moved.values().map(Vec::len).sum();
        let plural = |count: usize| if count == 1 { "" } else { "s" };
        match (total, self.failed.is_empty()) {
            (0, _) => "// Nothing was moved.\n".to_string(),
            (_, true) => format!(
                "// Alhamdulillah {total} requested path{} {} successfully refactored:\n",
                plural(total),
                if total == 1 { "was" } else { "were" }
            ),
            (_, false) => {
                let failed: usize = self.failed.iter().map(|group| group.files.len()).sum();
                format!(
                    "// Partly done: {total} requested path{} moved, {failed} failed. The groups that moved stay moved; every failed group was put back.\n",
                    plural(total)
                )
            }
        }
    }

    fn render_language(&self, response: &mut String, lang: &str, files: &Pairs) {
        response.push_str(&format!("\n// {} results:\n\n", capitalize(lang)));
        if lang == "typescript" {
            response.push_str(&format!(
                "// {} TypeScript/JavaScript source file{} moved (limit: {}).\n\n",
                self.typescript_source_count,
                if self.typescript_source_count == 1 {
                    ""
                } else {
                    "s"
                },
                super::typescript::MAX_FILES_PER_MOVE
            ));
        }
        for (src, tgt) in files {
            response.push_str(&format!("{} -> {}  \n", self.show(src), self.show(tgt)));
        }
        for note in self.notes.get(lang).into_iter().flatten() {
            response.push_str(&format!("\n// Note: {note}  \n"));
        }
        // For Go: report any files gopls moved collaterally beyond what was requested.
        if lang == "go" {
            let collaterals = detect_go_collaterals(files, self.root);
            if !collaterals.is_empty() {
                response.push_str(
                    "\n// Note — Go moves entire packages. \
                     The following files were also relocated as part of the package rename:  \n\n",
                );
                for path in collaterals {
                    response.push_str(&format!("{}  \n", self.show(&path.display().to_string())));
                }
            }
        }
    }

    fn render_failed(&self, response: &mut String) {
        response.push_str("\n// Failed:\n");
        for group in &self.failed {
            response.push_str(&format!(
                "\n// {} — {}\n\n",
                capitalize(&group.lang),
                group.error
            ));
            for (src, tgt) in &group.files {
                response.push_str(&format!("{} -> {}  \n", self.show(src), self.show(tgt)));
            }
        }
    }

    /// A path relative to the project root when it lies below it.
    fn show(&self, path: &str) -> String {
        match self.root {
            Some(root) => Path::new(path)
                .strip_prefix(root)
                .map(|relative| relative.display().to_string())
                .unwrap_or_else(|_| path.to_string()),
            None => path.to_string(),
        }
    }
}

fn capitalize(word: &str) -> String {
    let mut chars = word.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

#[cfg(test)]
mod tests;
