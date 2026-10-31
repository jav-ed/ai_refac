//! What code and configuration name by path when pages, pictures and code
//! move together: the code that imports a page or a picture the Markdown
//! driver moves (fixed first, `imports`), and the files that still name an old
//! path afterwards (listed, `still_mentioned`).

pub(in crate::logic) mod imports;
pub(in crate::logic) mod still_mentioned;
