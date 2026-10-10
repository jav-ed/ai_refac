//! `refac move --help`.

pub(in crate::cli) const ABOUT: &str = "Move or rename files and folders; imports, module lines, package lines and Markdown links follow.";

pub(in crate::cli) const LONG_ABOUT: &str = r#"Move or rename files and folders and rewrite every import, module line, package line and
Markdown link that pointed at them.

Give the files as paired --source-path / --target-path flags: the first source goes to the
first target, and so on. Paths may be absolute or relative to --project-path. A target may be
in a new folder (it is created). Several languages may be mixed in one call; refac groups the
files by language and moves each group in one go, with one language server per group.

--project-path is the PACKAGE root of the language, not the monorepo root:
  TypeScript/JavaScript  the folder with the tsconfig.json that includes every caller
  Kotlin/Android         the Gradle root (settings.gradle.kts)
  Go                     the folder with go.mod       Rust    the folder with Cargo.toml
  Python                 the project folder           Dart    the package folder with pubspec.yaml
  Markdown               any folder; links in every Markdown file below it are updated

RULES BY LANGUAGE
  TypeScript/JavaScript  Folders can be moved. At most 30 source files per call (the count of a
                         folder's files is included; the error tells the measured number). The tsconfig
                         decides which files are callers: files it does not include are not updated.
  Kotlin/Android         Needs the JetBrains Kotlin language server (REFAC_KOTLIN_SERVER; `refac doctor
                         kotlin`). Every call imports the Gradle build first (about 24 s), so put all
                         moves in ONE call. The package line, imports, usages and Android XML (manifest,
                         layouts, navigation graphs) follow. .java files, folders that contain Java, and
                         moves between modules or source sets are refused. A Kotlin Multiplatform build
                         (a commonMain, commonTest or <target>Main source set) is planned on a plain-JVM
                         copy of its source sets; a file that declares expect or actual is refused.
  Go                     Moving a .go file to another folder moves the WHOLE package (gopls renames the
                         package); the output lists the files that moved with it. A rename within the same
                         folder is a plain file rename.
  Rust                   A .rs file can be renamed inside its folder. To move a module to another place use
                         `refac move-module`; a cross-folder .rs target is refused and says so.
  Python                 Imports are rewritten by Rope (Pyrefly as fallback). Imports that go through an
                         `__init__.py` re-export may be missed: search for the old module name afterwards.
  Dart                   `dart pub get` must have run (package_config.json); a move that would leave a
                         `package:` import pointing at a missing file is refused before anything is written.
  Markdown               Files, images and other assets, and folders of them. Links (inline, reference,
                         HTML href/src/srcset, %20, ?query, #anchor) are rewritten, also the ones that point
                         at files other languages moved in the same call. Code blocks, comments and files
                         ignored by .gitignore are left alone.

WHAT YOU GET BACK
  A section per language with one `old -> new` line per moved path, then `// Note:` lines for what refac
  did not change (strings, build scripts, ProGuard rules) or changed beyond the request (Go package
  files). If one language's group fails while another succeeded, the successful groups stay moved and
  the failure is listed under `// Failed:`. Within one group the move is all or nothing: the files are
  put back and the message says so. Exit code 0 means every group moved.

PREVIEW (--dry-run)
  Plans the same move and changes no file: per language the paths that would move, then one
  `// <file> (N edits)` line per file that would be edited (an edit is one rewritten import, module
  line, package line or link), and the notes. It runs the same checks and refuses what the move
  refuses (exit 1, same report). Language servers start as for a real move and stop afterwards.
  TypeScript, Markdown, Dart, Go, Rust and Pyrefly are planned from the plan the tool makes before it
  writes; TypeScript also resolves every rewritten import against the files as they will be after the
  move (an import of an asset through an alias, which only the real move can check, is counted in a
  note). Python (Rope) and Kotlin cannot plan without moving, so they run the real move, with its own
  checks, on a throw-away copy of the project (limit 500 MiB, REFAC_DRY_RUN_COPY_MAX_MB; Rope copies
  only .py and .pyi files) and report the difference; every path must then lie inside the project.
  Go and Rust moves of several packages are planned one after the other; two plans that edit the same
  text are refused, with the advice to move them in separate commands."#;

pub(in crate::cli) const AFTER_LONG_HELP: &str = r#"EXAMPLES
  # One file
  refac move --project-path /my/project --source-path src/old/name.ts --target-path src/new/name.ts

  # Several files, paired in order, mixed languages
  refac move --project-path /my/project \
    --source-path src/a.ts --source-path docs/a.md \
    --target-path src/x.ts --target-path docs/x.md

  # A Kotlin file to another package: package line, imports and Android XML follow
  refac move --project-path /my/gradle/project \
    --source-path app/src/main/kotlin/com/example/ui/Home.kt \
    --target-path app/src/main/kotlin/com/example/home/Home.kt

  # A Markdown folder: every link into it and out of it follows
  refac move --project-path /my/docs --source-path guides --target-path handbook/guides

  # See what a move would do, change nothing
  refac move --dry-run --project-path /my/project --source-path src/old/name.ts --target-path src/new/name.ts

  # Machine-readable result
  refac move --json --project-path /my/project --source-path a.go --target-path pkg/a.go

JSON (--json): {"status":"ok","operation":"move","project_path":...,"source_path":[...],"target_path":[...],
"result":"<the text above>"}. With --dry-run it adds "dry_run":true, "moved_paths", "edited_files",
"edits", "files":[{"path","edits"}], "moves":[{"from","to"}] and "notes":[...] (paths relative to the
project path). A failure prints {"status":"error","error":"..."} on stderr and exits 1.
A language server that is missing is named with the places looked at: run `refac doctor <language>`."#;
