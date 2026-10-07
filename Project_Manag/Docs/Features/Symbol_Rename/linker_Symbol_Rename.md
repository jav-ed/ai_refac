# Symbol rename

`refac rename` renames one symbol (a variable, parameter, function, method, field, type, enum entry) and every place the language's own tooling links to it, in six languages with one command and one set of rules. The command picks the language from the file extension (`.ts` `.tsx` `.js` `.jsx` `.mts` `.cts` `.mjs` `.cjs`, `.kt`, `.go`, `.rs`, `.py`, `.dart`), asks that language's server for the references and the edits, proves in memory that the renamed program still means the same, and only then writes, with an undo log. A rename that cannot be proven writes nothing.

```bash
refac rename --project-path /path/to/project --file shape/shape.go \
  --symbol Area --new-name Surface            # add --dry-run to only look
```

Servers are never left running. Every call starts the language server it needs, uses it, stops it, and removes whatever it created, because an idle language server holds hundreds of megabytes. When the server is not installed, the error lists every place that was looked at and names `refac doctor <language>`, which prints the install steps, the environment variable, and a start check. That is the whole self-help loop: the agent runs one command, reads one error, runs the command it names, and tries again. See [Language servers](../../Setup/language_Servers.md).

## Languages

| Language | Server | Project root (`--project-path`) | What is special | Page |
| :--- | :--- | :--- | :--- | :--- |
| TypeScript / JavaScript | TypeScript 7 native language server (bundled, started by Bun) | folder with the authoritative `tsconfig.json` | its own engine, not the shared one | [TypeScript symbol rename](../TypeScript/symbol_Rename.md) |
| Kotlin (JVM, Android) | JetBrains Kotlin language server | Gradle root | a class moves with its file; Android XML follows | [Kotlin symbol rename](../Kotlin/symbol_Rename.md) |
| Go | gopls | folder with `go.mod` or `go.work` | refuses clashes itself; package rename is a move | [Go](go.md) |
| Rust | rust-analyzer | folder with `Cargo.toml` | does not rename inside `macro_rules!`; module rename is `move-module` | [Rust](rust.md) |
| Python | basedpyright | any folder (it reads `pyrightconfig.json` or `[tool.pyright]` there) | overrides renamed with the base method; untyped receivers are reported | [Python](python.md) |
| Dart | the Dart SDK's analysis server | folder with `pubspec.yaml` and `.dart_tool/package_config.json` | needs `dart pub get`; doc-comment links follow | [Dart](dart.md) |

Go, Rust, Python, and Dart (and Kotlin, ported onto it) share one engine: [Engine](engine.md) describes the steps, what each language contributes, and where the code lives.

## Choosing the symbol

Refac finds every whole-word match of `--symbol` in `--file` and asks the server what each one names. A match the server refuses (text in a string or comment, a keyword, a library symbol) is skipped; matches inside an earlier symbol's references are the same symbol. If one symbol remains it is renamed. If several remain (a field `total` and a local `total`), the command fails and lists each with its line and column, and the next call passes `--line` (1-based) and optionally `--column` (1-based byte column, like `rg --column`). If none remain it fails with the server's own reason.

## What every language gets

1. **Checks before any server starts.** The new name must be a plain identifier and not a keyword of the language, differ from the old one, and the file must be inside the project and have the right extension. Python refuses special methods such as `__init__`. A project that cannot work (no `go.mod`, no `Cargo.toml`, a Dart package without `dart pub get`) is refused with the fix.
2. **A plan without writing.** The server's references and edits for the symbol, plus the edits for anything the rename must carry along (see [Engine](engine.md): related symbols and override families). File creates, deletes, and renames in the server's answer are refused with advice: a package, module, or file name is a path and is changed with `refac move` or `refac move-module`.
3. **A proof in memory.** The server is shown the renamed text without saving it and asked for the references again. Every place that referred to the symbol must still refer to it (carried through the edits), every place the server listed must be edited, and no edit may lie outside the places that refer to the symbol (a few documented exceptions such as comments). A rename that captures another binding, or that makes a call reach a different declaration, loses or gains usages and is refused with the list of those places. A server that answers with only part of the edits (gopls under load) is asked again, up to a limit that depends on the language.
4. **The write.** Every planned file is re-read and must still read as it did when planning began; the edits go through an undo log and a failure restores everything.
5. **A report of what is left.** After the plan, refac scans the project's sources for the old name as a whole word and reports where it is still written: untyped receivers, text in strings and comments, other symbols with the same name, and for Rust the macro bodies the server never renames (printed as an `ATTENTION` note, because the build breaks until they are edited). The note ends with the `rg -w` command to check them.

## Output

Human text lists each changed file with its edit count, then the notes. `--json` returns `status`, `edits`, `edited_files`, `files`, and `notes`. `--dry-run` plans, proves, and reports the same, and writes nothing. Failures exit with code 1 and, with `--json`, an `error` string.

## Hard failures

Each stops the command and leaves every file unchanged: the server missing or broken (the error lists where it looked); a symbol the server will not rename; an ambiguous name; a name clash or shadowing the proof detects; a server answer that stays incomplete after the allowed attempts; an edit outside every reference of the symbol that the language does not expect; a file that changed during planning; a rename that needs a file operation.

## Related

- [Engine](engine.md): the steps in order, the `Language` and `RenameServer` interfaces, related symbols, override families, retries, the leftover scan, and the file map.
- [Language servers](../../Setup/language_Servers.md): `refac doctor`, the lookup order, environment variables, and what each server costs.
- [Symbol rename options](../../Investigation/symbol_Rename_Options.md): the evidence behind each server choice.
