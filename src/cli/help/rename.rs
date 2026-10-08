//! `refac rename --help`.

pub(in crate::cli) const ABOUT: &str = "Rename a symbol (variable, parameter, function, type, field, method) and every reference to it.";

pub(in crate::cli) const LONG_ABOUT: &str = r#"Rename a symbol (variable, parameter, function, type, field, method) and every reference to it.

Name the file, the symbol as it is written there, and the new name. The language comes from the file's
extension. A language server finds the symbol's real references (not a text search), so a different
symbol with the same name is left alone and one that is imported under another name is still found.

  Extensions                              Language         Done by
  .ts .tsx .js .jsx .mts .cts .mjs .cjs  TypeScript / JS   Oxc parser + TypeScript native language server
  .kt                                     Kotlin / Java    JetBrains Kotlin language server (+ Android XML)
  .go                                     Go               gopls
  .rs                                     Rust             rust-analyzer
  .py                                     Python           basedpyright
  .dart                                   Dart             Dart SDK analysis server

--project-path is the root the server loads (default: the current directory):
  TypeScript/JS  folder with the authoritative tsconfig.json    Kotlin  Gradle root (settings.gradle.kts)
  Go             folder with go.mod       Rust    Cargo package or workspace root
  Python         the pyright root         Dart    package folder with pubspec.yaml

WHEN THE NAME IS NOT UNIQUE
  If the name means several symbols in the file (a local `count` in two functions), nothing is
  renamed; the error lists every candidate with its line. Choose one with --line (1-based) and, when
  the line holds the name twice, --column (1-based byte column, like `rg --column`).

BEFORE ANYTHING IS WRITTEN
  1. The server plans the rename. 2. refac PROVES it: the server is shown the renamed text in memory
  and must find exactly the places it found before. A new name that clashes with or shadows another
  declaration changes what a call means; that is refused, naming the places, and no file changes.
  3. Only then the files are written, through an undo journal.
  Refused before a server even starts: a new name that is not a plain identifier, a keyword of the
  language, the current name itself, and for Python a dunder such as `__init__` (the interpreter calls
  it by its name). The name of a module or package is refused with the hint to use `refac move`.
  Python: methods that override the renamed one are found through basedpyright and renamed with it.

--dry-run does steps 1 and 2 and stops: the plan is printed, no file is touched.

SEVERAL RENAMES AT ONCE: --batch <FILE> (or `-` for stdin)
  A JSON list: [{"file": "src/a.rs", "symbol": "old_a", "new_name": "new_a"}, ...]; each entry may add
  "line" and "column". The language server is started ONCE (it takes seconds, Kotlin about 40) and
  stopped after the last rename. Entries run in order and each one sees the files as the one before
  left them, so a later entry can name a symbol by the name an earlier entry gave it. All or nothing:
  if entry 3 of 5 fails, entries 1 and 2 are taken back and the error names entry 3. One language per
  batch; TypeScript/JavaScript renames run one per command. --dry-run applies to the whole batch.

WHAT YOU GET BACK
  The `old -> new` line, then one `// path (N edits)` line per changed file and the totals. `// Note:`
  lines name places refac did NOT change and you may need to: the name inside strings and comments,
  untyped Python/Dart receivers, Rust `macro_rules!` bodies (marked ATTENTION: the build breaks until
  you edit them). Each note ends with the `rg -w` command that finds the rest.
  With --json: status, operation, project_path, file, symbol, new_name, dry_run, edits, edited_files,
  files [{path, edits}], notes. A batch prints operation "rename-batch" and a `renames` list.
  Exit code 0 on success, 1 on any failure (stderr says why and "Nothing was changed")."#;

pub(in crate::cli) const AFTER_LONG_HELP: &str = r#"EXAMPLES
  # A Go function and all of its callers
  refac rename --project-path /my/module --file shape/shape.go --symbol Area --new-name Surface

  # Look first: plan and prove, write nothing
  refac rename --project-path /my/crate --file src/lib.rs --symbol old_name --new-name new_name --dry-run

  # The name is used for two different locals: pick the one on line 12
  refac rename --file src/util.ts --symbol total --new-name grandTotal --line 12

  # Several renames in ONE server session (all or nothing)
  refac rename --project-path /my/crate --batch renames.json
  cat renames.json | refac rename --project-path /my/crate --batch -

  # Machine-readable
  refac rename --json --project-path /my/pkg --file lib/shapes.dart --symbol Circle --new-name Disc

A server that is missing is named with the places looked at: run `refac doctor <language>`."#;
