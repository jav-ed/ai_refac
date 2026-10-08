//! `move --dry-run`: for every backend the plan changes no file, and the real
//! move afterwards edits exactly the files the plan named and moves the paths
//! it listed. TypeScript and Markdown need only Bun; the others need their
//! language tool (`refac doctor <language>`).

mod dart;
mod go;
mod markdown;
mod python;
mod refusals;
mod rust;
mod typescript;
