# The shared rename engine

One engine, `src/drivers/lsp_rename/`, renames symbols for every language whose server speaks the Language Server Protocol: Go, Rust, Python, Dart, and Kotlin. The server finds the references and proposes the edits; the engine makes the answer safe to apply. A language is a small `Language` implementation that says what its identifiers look like, which server it starts, and the few edits it expects. TypeScript keeps its own engine ([TypeScript symbol rename](../TypeScript/symbol_Rename.md)) because its server and project loading differ; the request and report types and the occurrence scan are shared.

## One call, in order

`rename_symbol(language, request)` in `mod.rs`:

1. **Validate** the request (`names.rs`): new name is a plain identifier, not a keyword (`reserved_words`), not the old name, not refused by the language (`refuse_names`: Python special methods). `--column` needs `--line`.
2. **Project and file**: `language.project_root` (fails when the path is not a project of that language), then the file must exist inside it with a known extension.
3. **Occurrences**: whole-word matches of the symbol in the file, with line and column (`symbol_scan.rs`, shared with TypeScript).
4. **Start the server** (`language.start`), found by `src/servers/` (next section). It is stopped in step 8 whatever happens.
5. **Plan** (`discover.rs`): for each occurrence not already inside an earlier symbol's references, `prepareRename`, `references`, `rename`. Refusals are skipped; a refusal that names the symbol (a clash) beats the generic "not a symbol". Several distinct symbols are an ambiguity listing for `--line`. If the language opts in (`renames_overrides`), `family.rs` renames the overrides of the symbol too.
6. **Prove** (`related.rs`, `verify.rs`): see below. A failure the server may cause under load (`Unfaithful`) is retried up to `rename_attempts()` times (Go: 4) with the documents put back first.
7. **Settle**: after showing the server changed documents, `settle()` waits until its next answer reflects them (Dart needs it, see [Dart](dart.md)).
8. **Stop the server**, then plan the follow-ups: the language's own (Kotlin: Android XML) and the leftover scan (`leftovers.rs`).
9. **Write** (`apply.rs`, `journal.rs`): unless `--dry-run`. Each planned file is re-read first and must be unchanged; edits, file moves, and follow-up writes go through an undo log.

## The proof

The server does not notice a new name that collides with something in scope. The proof catches what it misses, using only what the server itself says:

- **Related groups** (`related.rs`). Normally every edit lies in a reference of the symbol. Some renames reach further on purpose (an interface method and its implementers; a struct field and the local tied to it by a shorthand initialiser). An edit outside the known references is accepted only if the server says that spot is itself a reference of another symbol, and that symbol becomes one more group. An edit that belongs to no symbol stops the rename. At most 50 groups.
- **Completeness**: every reference of a group whose text is the old name must be touched by an edit (an overlap is enough, because the Kotlin server sends letter-level edits). References spelled differently (`Self`, a Java accessor) are exempt.
- **Faithfulness** (`verify.rs`): the edited text is shown to the server in memory, the references of each renamed declaration are asked again, and the set of places must equal the old set carried through the edits. A usage lost or gained means a call now reaches another declaration; the error lists those places by line and column.
- **Expected exceptions** (`exempt_edits`): edits on comment lines (Go, Dart: a doc comment that mentions the symbol), and in Kotlin the symbol's own `import` line.

## What a language contributes

`language.rs` is the contract; each method has a default:

| Method | Meaning | Defaults to |
| :--- | :--- | :--- |
| `name`, `extensions`, `is_identifier_char`, `reserved_words` | what a name may be | required (Dart adds `$`) |
| `project_root` | the directory the server starts in; fails loudly | required |
| `start` | start the server on the project with the file open | required |
| `refuse_names` | names the language gives a meaning by spelling | none |
| `not_a_symbol_hint` | advice appended when the server says "not a symbol" | none |
| `renames_overrides` | ask the server for overrides and rename them too | false |
| `exempt_edits` | edits the language expects outside references | none |
| `rename_attempts` | tries when the answer fails the proof | 1 |
| `unrenamed_places` | text ranges the server never renames (Rust macro bodies) | none |
| `refuse_file_operations` | advice when the answer needs file creates, deletes, or renames; `None` lets files move (Kotlin classes) | "use `refac move`" |
| `follow_ups` | writes and notes beyond the server's edits | none |

`RenameServer` is the other contract: `request`, `sync_document`, `settle`, `shutdown`. `ProjectServer` implements it for Go, Rust, Python, and Dart over `lsp_session.rs`; the Kotlin server has its own start-up and implements it too. What differs per server (capabilities, how it announces that the project is loaded, whether it can list implementations) lives in `src/drivers/lsp_client/server.rs`.

## Finding the server

`src/servers/` owns the lookup, so a missing server is explained the same way everywhere (`rename`, `move`, `doctor`). Order: the language's environment variable (a wrong value is final, there is no fallback behind it), then `PATH`, then the folders installers use (`~/go/bin`, `~/.cargo/bin`, `~/.local/bin`, the project's `.venv/bin`). A candidate counts only when its version command runs (20 second limit), because a stand-in that exists and fails is worse than none. Every place looked at is recorded, and the error prints that record and `Run refac doctor <language>`. `catalog/` holds one file per server with its install steps; `doctor.rs` and `doctor/render.rs` print them; `handshake.rs` starts the server on an empty folder and stops it to prove it works (`refac doctor <language>` only).

## File map

- `src/drivers/lsp_rename/`: `mod.rs` (the call), `language.rs`, `server.rs`, `project_server.rs`, `names.rs`, `discover.rs`, `family.rs`, `related.rs`, `verify.rs`, `leftovers.rs`, `comments.rs`, `edits.rs` (parse a `WorkspaceEdit` into new file contents), `apply.rs` and `journal.rs` (write with undo), `languages/` (one file per language, Rust's macro scanner under `languages/rust/`), `test_language.rs` (a fake language and a fake server for unit tests).
- `src/servers/`: `mod.rs`, `locate.rs` and `locate/probe.rs`, `handshake.rs`, `doctor.rs`, `catalog/`.
- `src/cli/doctor.rs`: the `refac doctor` command. `src/logic/rename.rs`: picks the language from the extension. `src/logic/unavailable.rs`: the same explanation for a move whose driver is missing.
- Kotlin: `src/drivers/kotlin/rename.rs` is its `Language` implementation, `rename/imports.rs` its import exemption.

## Tests

Unit tests (normal `cargo test`) cover the proof (`verify/tests.rs`), related groups (`related/tests.rs`), override families (`family/tests.rs`), planning (`discover/tests.rs`), the retry logic (`tests.rs`), and each language's rules (`languages/tests.rs`), with a fake server that only knows words. The real-server scenarios are `#[ignore]`d and fail loudly, with refac's own missing-server explanation, when the server is not installed: `tests/go_rename.rs`, `tests/rust_rename.rs`, `tests/python_rename.rs`, `tests/dart_rename.rs`, `tests/kotlin_rename.rs`. Each ends by running the renamed project (build, `cargo check`, `dart analyze`, or the Python program and its checks), because a rename is right when the project still behaves. `tests/doctor.rs` runs the real binary on a machine with no server at all and needs nothing installed.
