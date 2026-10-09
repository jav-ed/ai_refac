# Symbol Rename

`refac rename` renames a TypeScript or JavaScript symbol (or, with `--batch`, several in one session) (variable, function, class, interface, enum, member) and updates every reference the type system links to it. It uses the TypeScript 7 native language server for the semantic part, and adds its own planning, verification, and hard-failure rules around it. Nothing is written until the whole rename has been planned and proven faithful in memory.

## Command

```bash
refac rename --project-path /path/to/package \
  --file src/lib/util.ts --symbol total --new-name grandTotal
```

- `--project-path`: the package root with the authoritative `tsconfig.json`. Defaults to the current directory, or `REFAC_PROJECT_PATH`.
- `--file`: a file that contains the symbol, absolute or relative to the project. It may be the declaration or any usage.
- `--symbol` and `--new-name`: the current and the new identifier.
- `--line` (1-based) and `--column` (1-based byte column, like `rg --column`): choose one occurrence when the name refers to several symbols in the file.
- `--dry-run`: plan and verify, report the edits, change no file.
- `--batch FILE` (or `-`): several renames of one project in one engine session, see below.
- `--json`: machine-readable result with `edits`, `edited_files`, and the per-file edit counts.

## Choosing the symbol

Refac finds every whole-word match of `--symbol` in the file and asks the engine what each one names. Matches inside strings and comments are not symbols and are dropped. If the remaining matches name one symbol, it is renamed. If they name several, for example a module-level `total` and a local `total` in one function, the command fails and lists each candidate with its line, column, source text, and edit count, so the next call can pass `--line` (and `--column`).

## What is updated

Declarations and usages across files: named, default, aliased, namespace, and type-only imports; `export ... from` re-exports; namespace member access (`util.total`); dynamic-import destructuring; class and interface members; enum members; JSX component names; and `.js` files in the project. TypeScript's own spelling rules apply:

- A shorthand property keeps its key: `{ total }` becomes `{ total: grandTotal }`.
- A re-export or a local export keeps its public name: `export { total } from "./x"` becomes `export { grandTotal as total } from "./x"`. Importers of the re-exporting module are unaffected.
- Text in strings, comments, JSON, and Markdown is never touched.

## Safety sequence

1. **Validate the request.** The new name must be a valid identifier, not a reserved word, and different from the old one. The file must be a TS/JS source inside the project, which needs a `tsconfig.json`.
2. **Load the project.** The engine lists the project's files first. Options TypeScript 7 removed make it ignore parts of the project silently, notably `baseUrl`, which breaks import resolution without an error. The engine reports such options as config errors, and Refac stops with that diagnostic. The file must also be in the listing.
3. **Plan without writing.** The engine returns every edit. Edits outside the project path (a referencing project) or inside `node_modules`, and any file create, rename, or delete, are rejected.
4. **Prove it in memory.** TypeScript's rename does not notice a new name that already exists in scope. A clash merges two declarations, and a shadowing name silently rebinds other usages while still compiling. Refac shows the engine the renamed text, asks for the references of the renamed declaration, and requires exactly one reference spelled with the new name per planned edit. A clash or capture changes that count and the rename is refused.
5. **Write.** Each file is re-read first and must be unchanged since planning. A failed write restores the files already written.

BOM and CRLF line endings are preserved. Positions are exchanged as UTF-8 byte offsets, so multibyte text is safe.

## Several renames

`refac rename --batch renames.json` takes a list of `{"file", "symbol", "new_name"}` entries (each may add `line` and `column`). The engine starts once (`src/drivers/typescript/rename/batch.rs`; a single rename is a batch of one). Each entry is planned and proven in memory against the files as the entries before it have written them, and then written, so a later entry may name a symbol by the name an earlier one gave it. The engine is stopped before the last write and on every failure. A failing entry puts back the earlier ones, newest first (`apply::revert`), and the error says which entry failed (`Rename 3 of 5 (a -> b in f) failed; the 2 earlier rename(s) were undone, so nothing was changed: ...`). With `--dry-run` nothing is written; each entry is planned on the text the earlier ones would write (an in-memory overlay), so the dry run accepts and refuses what the real batch does. A batch may not mix languages: it is refused before anything runs and names the first entry of another language. Tests: `tests/typescript/rename/batch.rs`.

## Hard failures

Each of these stops the command with a message and leaves every file unchanged: no `tsconfig.json`; a config that TypeScript 7 rejects; a file outside the tsconfig; an unrenameable symbol (standard library, `node_modules`, or not an identifier); an ambiguous name; a new name that clashes with or shadows another symbol; edits outside the project path; timeout (5 minutes) or engine memory above 4 GiB (`REFAC_TYPESCRIPT_MAX_RSS_MB`).

## Limits

- **Coverage:** only usages in projects the engine loads are renamed. Put every caller in the owning tsconfig, the same rule as file moves. A project that merely depends on the package, for example through `node_modules`, is not seen and is not updated. Search for the old name afterwards and run the project's typecheck.
- **Legacy configs:** projects whose tsconfig uses options TypeScript 7 removed (`baseUrl`, `moduleResolution: node10`) are rejected until the config is migrated. File moves are not affected, because they run on TypeScript 6.
- **Project references:** a rename from a referenced package is refused when the engine finds usages in a referencing package, because they lie outside the project path. Run it from a project path that owns both.
- **False refusals:** if the new name is already spelled as an alias of the same symbol (`import { total as grandTotal }`, renaming `total` to `grandTotal`), the count check refuses a legitimate rename. This is the safe direction.
- **Other languages:** rename is TypeScript and JavaScript only. Other languages could inshallah reuse the same planning and verification shape with their language servers.

## Why this engine

The engine comparison, the memory and speed numbers, and the failure modes found while testing are in [TypeScript rename engines](../../Investigation/typescript_Rename_Engines.md).
