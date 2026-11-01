# Languages

One file per language. Open the file of the language you are about to change before the first change in it, and leave the others unread: each holds that language's setup (tool, install command, `--project-path`), its limits and its rename rules. A language you do not touch needs neither its file nor its tools, and `refac doctor` listing it as `MISSING` needs no action.

The language of a file comes from its extension, and one `move` call may mix languages. A directory in any other language fails with a clear error.

| Language | Files | Directories | Logical modules | Symbols (`rename`) | `--project-path` |
|---|---|---|---|---|---|
| TypeScript / JavaScript, Astro, MDX | yes | yes | no | yes (`.astro` with `--astro`) | package root whose `tsconfig.json` includes every caller; TypeScript 7 must accept it (no `baseUrl`, no `moduleResolution: node10`) |
| Kotlin / Android | yes | yes | no | yes | Gradle root (`settings.gradle.kts`), not a module folder |
| Python | yes | no | no | yes | folder pyright treats as the root (`pyrightconfig.json` or `[tool.pyright]`) |
| Rust | yes | no | yes (`move-module`) | yes | folder with `Cargo.toml` (package or workspace) |
| Go | yes | no | no | yes | folder with `go.mod` or `go.work` |
| Dart | yes | no | no | yes | package folder with `pubspec.yaml`, after `dart pub get` |
| Markdown | yes | yes (folders of Markdown and assets) | no | no | - |

- [TypeScript, JavaScript, Astro, MDX](typescript.md): read before a TypeScript, JavaScript, Astro or MDX move or rename, and when one fails. The 30-file limit and tsconfig coverage, memory limit and rollback, reference-update gaps, how `.astro` and `.mdx` importers, `<style>` and `.css` paths, content-collection strings and `import.meta.glob` patterns follow a move, what `rename --astro` needs, and the TypeScript 7 rename rules.
- [Kotlin and Android](kotlin.md): read before the first Kotlin or Android move or rename, and whenever one fails or leaves the build broken. Why a single change is refused and how to batch, the `move --batch moves.json` list, what a move imports for the moved file and for the files left behind (operators, `get`, `for` loops, destructuring, `by`, other Gradle modules), what it leaves alone, a table that explains each reason a failed move prints (a killed server, a crash, an expired server build, a Gradle import error, a timeout), what is refused, multiplatform builds, rename, Android and the notes.
- [Rust](rust.md): read before a Rust move or rename. `move-module` (semantic module moves, what it resolves and what it refuses), ordinary same-folder `.rs` renames with `move`, and the symbol-rename rules (traits, `use` trees, `macro_rules!`).
- [Go](go.md): read before a Go move or rename. A cross-directory move renames the whole package, the `go.mod` requirement, and the symbol-rename rules.
- [Python](python.md): read before a Python move or rename. Rope for moves and basedpyright for rename, the `__init__.py` re-export limits of moves, and the symbol-rename rules (overrides, `__all__`, untyped receivers).
- [Dart](dart.md): read before a Dart move or rename. The package config that `package:` imports need (`dart pub get`) and the symbol-rename rules.
- [Markdown](markdown.md): read when moving Markdown files, pictures or documents, or when a link was not rewritten. Which links are rewritten and which are not; Markdown links also follow every move in the other languages.
