# Kotlin Symbol Rename

`refac rename` on a `.kt` file renames one Kotlin symbol (variable, property, function, class, interface, enum entry, member) and every reference the compiler links to it, including Java callers of Kotlin code and Java accessors of renamed properties. The JetBrains Kotlin language server does the semantic part. Refac plans every edit first, proves in memory that the rename keeps the program's meaning, repairs Android XML, and only then writes, with rollback. Nothing is written when any step fails.

## Command

```bash
refac rename --project-path /path/to/gradle/root \
  --file app/src/main/kotlin/com/example/util/Helper.kt \
  --symbol shout --new-name yell
```

- `--project-path`: the Gradle root (the folder with `settings.gradle(.kts)`). Defaults to the current directory, or `REFAC_PROJECT_PATH`.
- `--file`: a `.kt` file inside the project that contains the symbol, as declaration or as a usage.
- `--symbol` and `--new-name`: the current and the new identifier. The new name must be a plain identifier and not a Kotlin hard keyword (backtick names are not supported).
- `--line` (1-based) and `--column` (1-based byte column): choose one occurrence when the name refers to several symbols in the file.
- `--dry-run`: plan and verify, print what would change, write nothing.
- `--batch <file|->`: several renames in one call (a JSON list of `file`, `symbol`, `new_name`, optional `line` and `column`). The Gradle import, about 40 s, is paid once for all of them, and the whole batch is all or nothing. A class rename moves its file; a later entry names that file by its new path, because refac tells the server about the move (`kotlin/resync.rs`) before the next rename. See [Engine](../Symbol_Rename/engine.md).
- `--json`: machine-readable result with the per-file edit counts and `notes`.

The server must be installed first: see [Kotlin server setup](../../Setup/kotlin_Server.md); `refac doctor kotlin` checks the setup and prints what is missing. The steps below are the shared engine of Go, Rust, Python, Dart, and Kotlin ([Symbol rename](../Symbol_Rename/linker_Symbol_Rename.md), [Engine](../Symbol_Rename/engine.md)); this page is what is specific to Kotlin.

## Choosing the symbol

Refac finds every whole-word match of `--symbol` in the file and asks the server what each one names. A match the server refuses (text in a string or a comment, a keyword, a library symbol) is skipped. Matches inside the references of an earlier symbol are the same symbol. If exactly one symbol remains it is renamed. If several remain, for example a property `name` and a parameter `name`, the command fails and lists each candidate with its line, column, and edit count so the next call can pass `--line`. If none remain the command fails with the server's reason.

## Safety sequence

1. **Validate the request** before any server starts: valid new name, a `.kt` file inside the project, `--column` only with `--line`.
2. **Plan without writing.** For the chosen symbol refac collects its references before the rename and the server's edits. A file create or delete in the server's answer is a protocol surprise and stops the command.
3. **Prove it in memory.** The server does not notice a new name that collides with something in scope: the rename then compiles into a different program (a call that used to reach the extension reaches a member of the same name). Refac gives the server the renamed text without saving it, asks for the references of the renamed declaration again, and requires exactly the places that referred to it before, carried through the edits. A usage that was lost or gained means the meaning changed, and the rename is refused with the list of those places. A true redeclaration is refused by the server itself. Every edit must also lie inside a place that refers to the symbol, with one exception: the `import` line of the symbol itself, which the server lists no reference for (extension functions) but rewrites, or drops together with the blank line after it when the new name no longer needs the import.
4. **Repair Android.** Class names in manifests, layouts, and navigation graphs follow a renamed class ([Android layer](android_Layer.md)). Old names that refac does not edit are reported.
5. **Write.** Every planned file is re-read first and must still read as it did when the plan was made, because the server takes a while. Edits, XML changes, and the file rename go through an undo log; a failure restores everything.

## Classes are renamed with their file

Renaming a class that is the top-level type of a file named like it also renames the file (`Greeter.kt` becomes `Welcomer.kt`), like the IDE does. The output says so: `X is renamed to Y together with its class`. Usages in Kotlin, Java, and Android XML follow.

## Hard failures

Each stops the command and leaves every file unchanged: no Gradle root; the server missing, unready, or past the timeout (`REFAC_KOTLIN_TIMEOUT_SECS`, default 600); a symbol the server will not rename (library, keyword, string); an ambiguous name; a rename that would lose or gain usages (clash or shadowing); a target file that already exists; an edit outside every reference of the symbol; a file that changed while planning; unreadable Android XML or a missing Android namespace in a module that has a manifest.

## Limits

- **Coverage:** only code the Gradle import covers is renamed. Sources of modules outside the build, generated code, and text files are not. Search for the old name afterwards and build the project.
- **Not edited:** ProGuard and keep rules, build scripts, string literals, reflection, resource ids. The server refuses to rename a resource id, which is the safe answer. Names left behind in such files are listed in the notes.
- **False refusals:** a rename onto a name that is already an alias of the same symbol changes the usage count and is refused, which is the safe direction.
- **Cost:** about 30 seconds of Gradle import per call, plus the requests (one reference query per candidate symbol, one rename, one verification query).
- **Other languages:** TypeScript has its own rename ([TypeScript symbol rename](../TypeScript/symbol_Rename.md)). Both share the request and report types and the occurrence scan in `src/drivers/symbol/scan.rs`.
