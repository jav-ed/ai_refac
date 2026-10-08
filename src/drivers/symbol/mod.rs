//! What symbol rename shares across languages: the request and report types
//! (`rename`) and the whole-word scan of a source text for a symbol (`scan`).
//! TypeScript and the language-server engine in `lsp::rename` both build on it.

pub mod rename;
pub mod scan;
