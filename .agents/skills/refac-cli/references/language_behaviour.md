# Language-Specific Behaviour

Read this file when a move involves a language with non-obvious semantics or when a move is behaving unexpectedly.

## Go — whole-package moves

Moving any `.go` file cross-directory causes gopls to rename the **entire package**. All files in the source directory move together. If `pkg/` contains `a.go`, `b.go`, and `c.go`, asking to move `pkg/a.go` will cause all three to end up in the target directory. Partial-package moves are not supported.

Same-directory renames (file rename with no directory change) are a filesystem-only operation — gopls is not involved and no import paths change.

Requires `go.mod` at the project root for any cross-directory move. Without it the move will error.

## Rust — semantic module moves

Use logical module paths for any structural move:

```bash
refac move-module --project-path /path/to/cargo-workspace \
  crate::engine::matching crate::domain::matching
```

The command resolves the source with embedded rust-analyzer HIR, moves its complete file or `mod.rs` subtree, rewrites resolved references across the Cargo workspace, adjusts affected `super::` paths, creates conventional missing parent modules, and runs `cargo check --workspace --all-targets`. It never creates `#[path]` or compatibility re-export shims. A failed check rolls the planned source changes back.

Source and target must be in the same crate. A `crate::...` source that is ambiguous across workspace crates is rejected with the matching declaration locations. Workspace dependants of the selected crate are updated.

Strict v1 rejections include inline source modules, `#[path]`, attributed module declarations such as `#[cfg]`, visibility other than private/`pub`/`pub(crate)`, syntax errors, complex paths the rewriter cannot preserve, and grouped imports that would need restructuring.

Use ordinary `refac move` for same-directory `.rs` filename renames; rust-analyzer LSP rewrites the module symbol. Cross-directory `.rs` paths through `move` are rejected and direct you to `move-module`.

## Go, Rust, Python, Dart: symbol rename

`refac rename` works in these four with the same rules as TypeScript and Kotlin: the language comes from the extension, nothing is written until the rename is planned and proven in memory, a clash or shadowing is refused with the lost or gained usages listed, and `--dry-run` writes nothing. The server is started and stopped by the command. Per language:

- **Go** (gopls, `--project-path` has `go.mod` or `go.work`): renames interface methods together with their implementers and the test variants. Renaming a `package` clause is refused: use `refac move` on the package directory. gopls sometimes answers under load with part of the edits; refac notices and asks again by itself.
- **Rust** (rust-analyzer, `--project-path` has `Cargo.toml`): follows traits, `use` trees, and re-exports. Names written inside `macro_rules!` are not renamed; the output starts with `ATTENTION` and names the lines to edit by hand. A `mod` name is refused: use `refac move-module`.
- **Python** (basedpyright, any `--project-path`): renames overrides together with the base method (rename from the base, not from an override), `__init__.py` re-exports, `__all__` strings, and keyword arguments. Special methods (`__init__`) are refused. Calls on a parameter without an annotation are listed in the note, not renamed. A module name in an import is a path: use `refac move`.
- **Dart** (the SDK's `dart language-server`, `--project-path` has `pubspec.yaml` and `.dart_tool/package_config.json`): renames overrides, `export ... show`, named arguments, field formals, and `[Name]` doc links. Run `dart pub get` first or the command stops and says so.

**Several renames: `--batch`.** Two or more renames of one project go into one `refac rename --batch` call: the server starts once instead of once per rename (measured on the Rust fixture: four renames 18.8 s as four commands, 4.6 s as one batch, same files), each rename is proven and written against the files the previous one left, and the batch is all or nothing. Kotlin gains the most (about 40 s per server start). A server is deliberately not kept running between commands: its view of the files would go stale when anything else edits them, and it would hold 0.2-2 GB of memory. One language and one project per batch; TypeScript/JavaScript takes one rename per command.

When the server is not installed the error says what was looked at; `refac doctor <language>` prints the install steps. Pages: [Symbol rename](../../../../Project_Manag/Docs/Features/Symbol_Rename/linker_Symbol_Rename.md), [Language servers](../../../../Project_Manag/Docs/Setup/language_Servers.md).

## Dart — package URI rewriting requires package config

`package:` URI imports are only rewritten if `.dart_tool/package_config.json` exists at the project root. Without it, a move that would leave a `package:` import pointing at a missing file is refused before anything is written, and the error lists the imports.

Run `dart pub get` in the project root to generate it before calling `refac`.

## TypeScript / JavaScript — tsconfig coverage

Point `--project-path` at the package containing the authoritative `tsconfig.json`. Its `include` or `files` configuration must cover all local TS/JS sources that participate in imports. External packages in `node_modules` do not need to be included.

Refac scans the complete tsconfig source set with Oxc and resolves code imports with TypeScript without constructing a type checker. Batches remain limited to 30 contained source files. Ordinary apply/verification failures roll back; a crash or forced termination can interrupt rollback.

The helper is terminated and reaped after 5 minutes or sampled RSS above 4 GiB (100 ms sampling). `REFAC_TYPESCRIPT_MAX_RSS_MB` changes the threshold in positive integer MiB. Inspect the working tree after a limit failure. One-file batches still scan every configured source; do not shrink coverage to hide callers.

### Reference-update gaps

Declared aliases are rewritten and verified. Project references, symlink moves, overlapping requests, and ambiguous locally bound `require` calls fail explicitly. Computed imports, arbitrary path strings, comments, and package/config metadata need manual audit. Search old paths and run the project build. See [TypeScript backend](../../../../Project_Manag/Docs/Features/TypeScript/linker_TypeScript.md) for supported forms and exact limits.

## TypeScript / JavaScript — symbol rename

`refac rename` uses the TypeScript 7 native language server (installed from the locked `typescript-native` dependency by the first run's `bun install --frozen-lockfile`). It is a separate engine from file moves.

- Pick the symbol with `--file` and `--symbol`. If the name refers to several symbols in that file, the command lists the candidates; repeat it with `--line` (1-based) and, if one line has several, `--column` (1-based byte column).
- Nothing is written until the rename is planned and verified in memory. A new name that clashes with or shadows another symbol is refused, even when the result would still compile.
- TypeScript keeps public names: `{ total }` becomes `{ total: renamed }` and `export { total } from "./x"` becomes `export { renamed as total } from "./x"`. Strings, comments, JSON, and Markdown are never edited.
- A tsconfig that uses options TypeScript 7 removed (`baseUrl`, `moduleResolution: node10`) is rejected with the engine's diagnostic, because the engine would otherwise miss files silently. File moves are unaffected.
- Only usages in projects the engine loads are renamed. Put every caller in the package's tsconfig, then search for the old name and run the project's typecheck. Edits that would land outside `--project-path` (for example a referencing project) stop the rename.
- Use `--dry-run --json` first to see which files change and how many edits each gets.

Details, the safety sequence, and the evidence behind the engine choice: [TypeScript symbol rename](../../../../Project_Manag/Docs/Features/TypeScript/symbol_Rename.md).

## Python — re-export limits

Rope cannot trace imports that go through `__init__.py` re-exports. If a package re-exports a symbol and callers import via that re-export, those callers are not updated.

Namespace packages (directories with no `__init__.py`) may also see incomplete updates.

## Markdown

`move` takes Markdown files (`.md`, `.markdown`, `.mdx`), images and other assets (png, jpg, svg, pdf, fonts, media, zip), and folders that hold only those. Every link that follows is rewritten: inline links and images, reference definitions (also multi-line), angle-bracket and `%20` destinations, `?query` and `#fragment`, folder links, and the `href`/`src`/`srcset` values of raw HTML. A link is written the way the author wrote it (no forced `./`).

The same pass runs after any other backend: when `move` has moved a TypeScript, Python, Rust, Go, Dart, or Kotlin file or folder, the Markdown links to it are fixed too, and the response shows a `// Markdown links to the moved files:` note with the counts.

Not rewritten: web addresses, `#anchors`, `/site-root` paths, wiki-links `[[Page]]`, MDX imports, front matter, code blocks and spans, HTML comments, and paths inside other file types. `.gitignore`d files, `.git/`, and `node_modules/` are not searched. Markdown files that are not valid UTF-8 are named in the response and left unchanged. A folder with code of another language is refused.

Details: [Markdown](../../../../Project_Manag/Docs/Features/Markdown/linker_Markdown.md).

## Kotlin and Android

`move` and `rename` on `.kt` files use the JetBrains Kotlin language server; `--project-path` is the **Gradle root** (the folder with `settings.gradle(.kts)`), not a module folder. Install steps and `REFAC_KOTLIN_SERVER`: [Kotlin server setup](../../../../Project_Manag/Docs/Setup/kotlin_Server.md). Without the server every Kotlin call fails with those steps; refac never downloads it.

- **Moves:** a `.kt` file or a directory containing `.kt` files, inside `src/<set>/kotlin` or `src/<set>/java`. The `package` line, imports, and usages that relied on sharing a package are updated, Java callers too. A new directory plus a new name is done as move then rename; renaming a file also renames its class. All moves of one call share one server session, so batch them: every call costs about 30 seconds of Gradle import.
- **Refused before anything starts:** `.java` files, directories that contain Java, moves between modules or source sets, a target that exists, a target outside a source root. Everything else that fails rolls back to the original state and the error says so.
- **Rename:** pick the symbol with `--file` and `--symbol`; an ambiguous name lists candidates for `--line`/`--column`. A rename that would clash with or shadow another declaration is refused because the usage sets before and after differ. Renaming a class that names its file renames the file too. `--dry-run` writes nothing.
- **Android:** class names in the manifest, layouts, and navigation graphs follow moved and renamed classes, and `import <namespace>.R` / `BuildConfig` is added where a moved file needs it. A module with a manifest but no `namespace` in its build script is an error.
- **Reported, not edited:** old class names in ProGuard rules, build scripts (`mainClass`), service lists, configuration, and string literals appear as `// Note:` lines. Search for them and run `./gradlew compileKotlin`.

Details: [Kotlin and Android](../../../../Project_Manag/Docs/Features/Kotlin/linker_Kotlin.md).
