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

SAFETY
  The layout must be the conventional one; an ambiguous or unsupported layout is refused before
  anything is written ("strict v1"). After the move the workspace is checked again; if the check
  fails every changed file is put back. Cross-crate moves are not supported.

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
