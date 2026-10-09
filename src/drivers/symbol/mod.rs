//! What symbol rename shares across languages: the request and report types
//! (`rename`) and the whole-word scan of a source text for a symbol (`scan`).
//! TypeScript and the language-server engine in `lsp::rename` both build on it.
//! A dry run of a batch reads the files through `view`, which holds what the
//! earlier renames of the batch would have written.

pub mod batch;
pub mod rename;
pub mod scan;
pub mod view;
