# The shared rename engine

One engine, `src/drivers/lsp_rename/`, renames symbols for every language whose server speaks the Language Server Protocol: Go, Rust, Python, Dart, and Kotlin. The server finds the references and proposes the edits; the engine makes the answer safe to apply. A language is a small `Language` implementation that says what its identifiers look like, which server it starts, and the few edits it expects. TypeScript keeps its own engine ([TypeScript symbol rename](../TypeScript/symbol_Rename.md)) because its server and project loading differ; the request and report types and the occurrence scan are shared.

## One call, in order

`rename_symbol(language, request)` in `mod.rs` is a batch of one (`rename_symbols`, next section). Its steps:

1. **Validate** the request (`names.rs`): new name is a plain identifier, not a keyword (`reserved_words`), not the old name, not refused by the language (`refuse_names`: Python special methods). `--column` needs `--line`.
2. **Project and file**: `language.project_root` (fails when the path is not a project of that language), then the file must exist inside it with a known extension.
3. **Occurrences**: whole-word matches of the symbol in the file, with line and column (`symbol_scan.rs`, shared with TypeScript).
4. **Start the server** (`language.start`), found by `src/servers/` (next section). It is stopped in step 8 whatever happens.
5. **Plan** (`discover.rs`): for each occurrence not already inside an earlier symbol's references, `prepareRename`, `references`, `rename`. Refusals are skipped; a refusal that names the symbol (a clash) beats the generic "not a symbol". Several distinct symbols are an ambiguity listing for `--line`. If the language opts in (`renames_overrides`), `family.rs` renames the overrides of the symbol too.
6. **Prove** (`related.rs`, `verify.rs`): see below. A failure the server may cause under load (`Unfaithful`) is retried up to `rename_attempts()` times (Go: 4) with the documents put back first.
7. **Settle**: after showing the server changed documents, `settle()` waits until its next answer reflects them (Dart needs it, see [Dart](dart.md)).
8. **Stop the server**, then plan the follow-ups: the language's own (Kotlin: Android XML) and the leftover scan (`leftovers.rs`).
9. **Write** (`apply.rs`, `journal.rs`): unless `--dry-run`. Each planned file is re-read first and must be unchanged; edits, file moves, and follow-up writes go through an undo log.

## Several renames in one session (batch)

`rename_symbols(language, requests)` runs `batch.rs`. A server start is the largest fixed cost of a rename (Kotlin about 40 s and 1.6 GB, a large Rust project 30 s and 1.9 GB, Go 2-8 s, Python and Dart 1-2 s), so a series of renames of one project can share one start without any server staying resident between commands:

1. **Check every request first**, before a server exists: names, `--column` needs `--line`, one project root, all dry runs or none. The first request is also located (file, symbol, position), so a typo there costs no start. A batch of one keeps its plain, unprefixed error messages.
2. **Start the server once**, then for each request in order: locate it in the file as the previous request left it, plan, prove, write.
3. **Each rename sees the files as the previous one wrote them.** After a write, `RenameServer::after_apply(edited, moves)` tells a server that keeps its own view of the project (Kotlin: `didChangeWatchedFiles` created/changed/deleted plus the new texts, `kotlin/resync.rs`) what changed, so a later request may name a Kotlin file by the path its class rename moved it to. The other servers already hold the written texts, because the proof showed them. Dry-run entries write nothing, so after each one the documents are put back (`put_back`) and the next one is planned against the disk again.
4. **All or nothing.** Every write keeps its `Journal` (`apply_undoable`). A failing request undoes the journals of the earlier ones, newest first, stops the server, and says so: `Rename 3 of 5 (a -> b in f) failed; the 2 earlier rename(s) were undone, so nothing was changed: <reason>`. If an undo itself fails, the error says the project is half refactored and lists what to inspect with git.
5. **The server is stopped before the last write**, as for a single rename, and on every failure.

The CLI is `refac rename --batch <file|->` (`src/cli/rename.rs`), the dispatch is `handle_rename_batch` in `src/logic/rename.rs` (one language per batch; TypeScript/JavaScript keeps its own engine and takes one rename per command).

**Why no keep-alive server.** A server that stays running between commands would save the same starts, but: it holds its memory for as long as it lives (the design rule is that nothing stays resident); the files can change underneath it (an editor, `git checkout`, another tool) and the in-memory proof is only as good as its view of them, so every request would need a revalidation of every file; and it needs process management (a socket, a time-to-live, cleanup after a crash, one per project and language). The batch gets the saving that matters (all renames of one task) with none of that. If it is ever reconsidered, the case is Kotlin only, opt-in, with a time-to-live, a revalidation of changed files, and an explicit memory message in the output.

## The proof

The server does not notice a new name that collides with something in scope. The proof catches what it misses, using only what the server itself says:

- **Related groups** (`related.rs`). Normally every edit lies in a reference of the symbol. Some renames reach further on purpose (an interface method and its implementers; a struct field and the local tied to it by a shorthand initialiser). An edit outside the known references is accepted only if the server says that spot is itself a reference of another symbol, and that symbol becomes one more group. An edit that belongs to no symbol stops the rename. At most 50 groups.
- **Completeness**: every reference of a group whose text is the old name must be touched by an edit (an overlap is enough, because the Kotlin server sends letter-level edits). References spelled differently (`Self`, a Java accessor) are exempt.
- **Faithfulness** (`verify.rs`): the edited text is shown to the server in memory, the references of each renamed declaration are asked again, and the set of places must equal the old set carried through the edits. A usage lost or gained means a call now reaches another declaration; the error lists those places by line and column.
- **Outside the project.** A listed reference in a file outside the project folder that the server did not edit (a dependency in the package cache that uses the symbol) stops the rename with its own message and is not retried: no server edits those files, so the rename could not be complete.
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

`RenameServer` is the other contract: `request`, `sync_document`, `settle`, `after_apply` (batches), `shutdown`. `ProjectServer` implements it for Go, Rust, Python, and Dart over `lsp_session.rs`; the Kotlin server has its own start-up and implements it too. What differs per server (capabilities, how it announces that the project is loaded, whether it can list implementations) lives in `src/drivers/lsp_client/server.rs`.

## Finding the server

`src/servers/` owns the lookup, so a missing server is explained the same way everywhere (`rename`, `move`, `doctor`). Order: the language's environment variable (a wrong value is final, there is no fallback behind it), then `PATH`, then the folders installers use (`~/go/bin`, `~/.cargo/bin`, `~/.local/bin`, the project's `.venv/bin`). A candidate counts only when its version command runs (20 second limit), because a stand-in that exists and fails is worse than none. Every place looked at is recorded, and the error prints that record and `Run refac doctor <language>`. `catalog/` holds one file per server with its install steps; `doctor.rs` and `doctor/render.rs` print them; `handshake.rs` starts the server on an empty folder and stops it to prove it works (`refac doctor <language>` only).

## File map

- `src/drivers/lsp_rename/`: `mod.rs` (the call), `batch.rs` (several renames in one session), `language.rs`, `server.rs`, `project_server.rs`, `names.rs`, `discover.rs`, `family.rs`, `related.rs`, `verify.rs`, `leftovers.rs`, `comments.rs`, `edits.rs` (parse a `WorkspaceEdit` into new file contents), `apply.rs` and `journal.rs` (write with undo), `languages/` (one file per language, Rust's macro scanner under `languages/rust/`), `test_language.rs` (a fake language and a fake server for unit tests).
- `src/servers/`: `mod.rs`, `locate.rs` and `locate/probe.rs`, `handshake.rs`, `doctor.rs`, `catalog/`.
- `src/cli/doctor.rs`: the `refac doctor` command. `src/logic/rename.rs`: picks the language from the extension. `src/logic/unavailable.rs`: the same explanation for a move whose driver is missing.
- Kotlin: `src/drivers/kotlin/rename.rs` is its `Language` implementation, `rename/imports.rs` its import exemption, `resync.rs` tells its server what an earlier rename of a batch wrote.
- `src/cli/rename.rs`: `refac rename`, single and `--batch`.

## Tests

`tests/rename/encoding.rs` renames with each real server in files that have CRLF line endings and wide characters (an emoji, Japanese, an accent) before the symbol on the same line, and checks that the project still builds and that the text and every line ending are intact. Unit tests (normal `cargo test`) cover the proof (`verify/tests.rs`), related groups (`related/tests.rs`), override families (`family/tests.rs`), planning (`discover/tests.rs`), the retry logic (`tests.rs`), and each language's rules (`languages/tests.rs`), with a fake server that only knows words. The real-server scenarios are `#[ignore]`d and fail loudly, with refac's own missing-server explanation, when the server is not installed: `tests/rename/go.rs`, `tests/rename/rust.rs`, `tests/rename/python.rs`, `tests/rename/dart.rs`, `tests/kotlin/rename.rs`. Each ends by running the renamed project (build, `cargo check`, `dart analyze`, or the Python program and its checks), because a rename is right when the project still behaves. `batch/tests.rs` covers the batch with the word server (one start and one stop for several renames, a rename sees the previous one's write, a failure undoes the earlier renames, dry runs are independent, refusals before any start), `tests/rename/batch.rs` runs batches with the real Go, Rust, Python, and Dart servers (one rename by the name an earlier one gave, a failing batch leaves the project byte for byte as it was, independent dry runs), `tests/kotlin/rename.rs` adds a batch whose second rename names the file by the path the first one moved it to, and `tests/cli/batch_rename.rs` checks what the command line refuses without any server. `tests/cli/doctor.rs` runs the real binary on a machine with no server at all and needs nothing installed.
