//! `refac move-module --help`.

pub(in crate::cli) const ABOUT: &str =
    "Move a complete Rust module subtree and rewrite every path that reaches it.";

pub(in crate::cli) const LONG_ABOUT: &str = r#"Move a complete Rust module subtree and rewrite every path that reaches it.

A Rust module is a place in the crate's tree (`crate::engine::matching`), not just a file. This
command moves the module together with its children (the file or folder, `mod.rs` or the
`name.rs` + `name/` pair), changes the `mod` declarations, and rewrites every `use`, `crate::`,
`super::` and `self::` path that reached it, in every crate of the workspace, using rust-analyzer's
own semantic model. No `#[path]` shims are added.

ARGUMENTS
  <SOURCE_MODULE>  the module as it is now, starting with `crate::`
  <TARGET_MODULE>  the module as it should be, starting with `crate::`, in the SAME crate

--project-path is a Cargo package or a workspace root (default: the current directory).

WHAT IT REWRITES
  `crate::`, `super::` and `self::` paths, names imported with `use` (they keep their short form),
  paths in generic arguments, code behind `#[cfg(test)]`, and paths in macro arguments in every
  spelling (`vec![old::Item { .. }]`, `assert_eq!(super::old::f(), 1)`, `$crate::old::f()`). A module that leaves a grouped import (`use super::{a, old}`)
  gets an import of its own. The `mod` line keeps its reach: a private module moved deeper is
  declared `pub(super)` or `pub(in crate::parent)`, so its old users still see it.

SAFETY
  The layout must be the conventional one; an ambiguous or unsupported layout is refused before
  anything is written ("strict v1"): inline modules, `#[path]` and `#[macro_use]` declarations,
  attributes other than conditions and lints, `pub(in ..)` that is not an absolute `crate::` path.
  After the move the workspace is checked again (`cargo check --workspace --all-targets`, the
  slow part: about 4 minutes on this repository); if the check fails every changed file is put
  back. Cross-crate moves are not supported.

WHAT YOU GET BACK
  `// Alhamdulillah Rust module moved semantically:` then `old -> new`, the number of filesystem
  paths moved and source files updated. With --json: status, operation, project_path,
  source_module, target_module, moved_paths, edited_files."#;

pub(in crate::cli) const AFTER_LONG_HELP: &str = r#"EXAMPLES
  # Move a module (and everything below it) to another parent
  refac move-module --project-path /my/cargo-workspace crate::engine::matching crate::domain::matching

  # Same, from inside the workspace, machine-readable
  refac move-module --json crate::util::text crate::text

If rust-analyzer is missing: `refac doctor rust`. To rename a file inside its folder use `refac move`;
to rename a symbol use `refac rename`."#;
