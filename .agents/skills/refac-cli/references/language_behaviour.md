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

## Dart — package URI rewriting requires package config

`package:` URI imports are only rewritten if `.dart_tool/package_config.json` exists at the project root. Without it, only relative imports are updated.

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

Only relative links are rewritten. Absolute URLs and `http://` / `https://` links are left unchanged.

Links inside fenced code blocks and inline code spans are not rewritten.

## Kotlin and Android

`move` and `rename` on `.kt` files use the JetBrains Kotlin language server; `--project-path` is the **Gradle root** (the folder with `settings.gradle(.kts)`), not a module folder. Install steps and `REFAC_KOTLIN_SERVER`: [Kotlin server setup](../../../../Project_Manag/Docs/Setup/kotlin_Server.md). Without the server every Kotlin call fails with those steps; refac never downloads it.

- **Moves:** a `.kt` file or a directory containing `.kt` files, inside `src/<set>/kotlin` or `src/<set>/java`. The `package` line, imports, and usages that relied on sharing a package are updated, Java callers too. A new directory plus a new name is done as move then rename; renaming a file also renames its class. All moves of one call share one server session, so batch them: every call costs about 30 seconds of Gradle import.
- **Refused before anything starts:** `.java` files, directories that contain Java, moves between modules or source sets, a target that exists, a target outside a source root. Everything else that fails rolls back to the original state and the error says so.
- **Rename:** pick the symbol with `--file` and `--symbol`; an ambiguous name lists candidates for `--line`/`--column`. A rename that would clash with or shadow another declaration is refused because the usage sets before and after differ. Renaming a class that names its file renames the file too. `--dry-run` writes nothing.
- **Android:** class names in the manifest, layouts, and navigation graphs follow moved and renamed classes, and `import <namespace>.R` / `BuildConfig` is added where a moved file needs it. A module with a manifest but no `namespace` in its build script is an error.
- **Reported, not edited:** old class names in ProGuard rules, build scripts (`mainClass`), service lists, configuration, and string literals appear as `// Note:` lines. Search for them and run `./gradlew compileKotlin`.

Details: [Kotlin and Android](../../../../Project_Manag/Docs/Features/Kotlin/linker_Kotlin.md).
