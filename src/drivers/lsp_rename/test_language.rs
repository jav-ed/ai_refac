//! A language with no server, for the tests of the engine's own pieces.

use super::language::{Language, UnrenamedPlace};
use super::server::RenameServer;
use anyhow::{Result, bail};
use async_trait::async_trait;
use std::path::{Path, PathBuf};

pub struct Plain;

#[async_trait]
impl Language for Plain {
    fn name(&self) -> &'static str {
        "Plain"
    }

    fn extensions(&self) -> &'static [&'static str] {
        &["pl", "plx"]
    }

    fn reserved_words(&self) -> &'static [&'static str] {
        &["class", "def"]
    }

    fn project_root(&self, project_path: &Path) -> Result<PathBuf> {
        Ok(project_path.to_path_buf())
    }

    /// Text between `<<` and `>>` is a place the server never renames.
    fn unrenamed_places(&self, text: &str) -> Vec<UnrenamedPlace> {
        let mut places = Vec::new();
        let mut from = 0;
        while let Some(open) = text[from..].find("<<") {
            let start = from + open + 2;
            let Some(close) = text[start..].find(">>") else {
                break;
            };
            places.push(UnrenamedPlace {
                start,
                end: start + close,
                what: "a template",
            });
            from = start + close;
        }
        places
    }

    async fn start(&self, _root: &Path, _file: &Path) -> Result<Box<dyn RenameServer>> {
        bail!("the Plain language has no server")
    }
}

/// A language server that knows words and nothing else: the symbol at a
/// position is the identifier written there, its references are every
/// whole-word match of it in the documents it was shown, and a rename edits
/// all of them. Tests make it misbehave the way real servers do: answer a
/// rename with only the first edit, or add an edit somewhere else.
pub struct WordServer {
    pub docs: std::collections::HashMap<PathBuf, String>,
    /// This many renames, from the first, answer with their first edit only.
    pub partial_renames: usize,
    pub renames_asked: usize,
    /// Edits every rename adds: (file, line, start column, end column).
    pub extra_edits: Vec<(PathBuf, u32, u32, u32)>,
    /// The reason every rename is refused with, after the symbol was found.
    pub refusal: Option<String>,
}

impl WordServer {
    pub fn new(files: &[(&Path, &str)]) -> Self {
        Self {
            docs: files
                .iter()
                .map(|(path, text)| (path.to_path_buf(), text.to_string()))
                .collect(),
            partial_renames: 0,
            renames_asked: 0,
            extra_edits: Vec::new(),
            refusal: None,
        }
    }

    fn is_word(character: char) -> bool {
        character == '_' || character.is_alphanumeric()
    }

    /// The word at the request position: its text and byte range.
    fn word_at(&self, params: &serde_json::Value) -> Option<(String, usize, usize)> {
        let path = uri_path(params["textDocument"]["uri"].as_str()?)?;
        let text = self.docs.get(&path)?;
        let position: lsp_types::Position =
            serde_json::from_value(params["position"].clone()).ok()?;
        let at = crate::drivers::lsp_text::TextIndex::new(text)
            .offset(position)
            .ok()?;
        let start = text[..at]
            .rfind(|c| !Self::is_word(c))
            .map_or(0, |i| i + text[i..].chars().next().unwrap().len_utf8());
        let end = at
            + text[at..]
                .find(|c| !Self::is_word(c))
                .unwrap_or(text.len() - at);
        (start < end).then(|| (text[start..end].to_string(), start, end))
    }

    /// Every whole-word match of `word`, as (file, byte start, byte end).
    fn matches(&self, word: &str) -> Vec<(PathBuf, usize, usize)> {
        let mut found = Vec::new();
        let mut paths: Vec<&PathBuf> = self.docs.keys().collect();
        paths.sort();
        for path in paths {
            let text = &self.docs[path];
            let mut from = 0;
            while let Some(at) = text[from..].find(word) {
                let (start, end) = (from + at, from + at + word.len());
                let before = text[..start].chars().next_back();
                let after = text[end..].chars().next();
                if !before.is_some_and(Self::is_word) && !after.is_some_and(Self::is_word) {
                    found.push((path.clone(), start, end));
                }
                from = end;
            }
        }
        found
    }

    fn range(&self, path: &Path, start: usize, end: usize) -> lsp_types::Range {
        let index = crate::drivers::lsp_text::TextIndex::new(&self.docs[path]);
        lsp_types::Range {
            start: index.position(start),
            end: index.position(end),
        }
    }
}

fn uri_path(uri: &str) -> Option<PathBuf> {
    url::Url::parse(uri).ok()?.to_file_path().ok()
}

#[async_trait]
impl RenameServer for WordServer {
    async fn request(
        &mut self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        use serde_json::json;
        let Some((word, ..)) = self.word_at(&params) else {
            return match method {
                "textDocument/prepareRename" => Ok(serde_json::Value::Null),
                _ => Err(crate::drivers::lsp_session::RpcError {
                    code: -32602,
                    message: "no identifier found".to_string(),
                }
                .into()),
            };
        };
        let uri = |path: &Path| url::Url::from_file_path(path).unwrap().to_string();
        match method {
            "textDocument/prepareRename" => Ok(json!({ "placeholder": word })),
            "textDocument/references" => Ok(json!(
                self.matches(&word)
                    .into_iter()
                    .map(|(path, start, end)| json!({
                        "uri": uri(&path),
                        "range": self.range(&path, start, end),
                    }))
                    .collect::<Vec<_>>()
            )),
            "textDocument/rename" => {
                self.renames_asked += 1;
                if let Some(message) = &self.refusal {
                    return Err(crate::drivers::lsp_session::RpcError {
                        code: -32803,
                        message: message.clone(),
                    }
                    .into());
                }
                let new_name = params["newName"].as_str().unwrap().to_string();
                let mut matches = self.matches(&word);
                if self.renames_asked <= self.partial_renames {
                    matches.truncate(1);
                }
                let mut changes: serde_json::Map<String, serde_json::Value> = Default::default();
                let mut push = |path: &Path, range: lsp_types::Range| {
                    changes
                        .entry(uri(path))
                        .or_insert_with(|| json!([]))
                        .as_array_mut()
                        .unwrap()
                        .push(json!({ "range": range, "newText": new_name }));
                };
                for (path, start, end) in matches {
                    push(&path, self.range(&path, start, end));
                }
                for (path, line, from, to) in &self.extra_edits {
                    push(
                        path,
                        lsp_types::Range {
                            start: lsp_types::Position::new(*line, *from),
                            end: lsp_types::Position::new(*line, *to),
                        },
                    );
                }
                Ok(json!({ "changes": changes }))
            }
            other => panic!("the word server does not answer {other}"),
        }
    }

    async fn sync_document(&mut self, path: &Path, text: &str) -> Result<()> {
        self.docs.insert(path.to_path_buf(), text.to_string());
        Ok(())
    }

    async fn shutdown(self: Box<Self>) {}
}
