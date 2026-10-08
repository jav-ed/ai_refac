# Symbol Rename Options

Which language server renames a symbol in Go, Rust, Python, and Dart, and what had to be added around each so that a rename is applied only when it can be proven safe. This document records what was run on 2026-10-07, on which cases, with which numbers, and which silent failures surfaced. Open it before changing a server choice, before adding a language to the shared engine, or before trusting a server's rename without the proof.

**Status: decided and implemented.** The engine is described in [Engine](../Features/Symbol_Rename/engine.md); the per-language pages are [Go](../Features/Symbol_Rename/go.md), [Rust](../Features/Symbol_Rename/rust.md), [Python](../Features/Symbol_Rename/python.md), and [Dart](../Features/Symbol_Rename/dart.md). TypeScript and Kotlin were studied earlier ([TypeScript rename engines](typescript_Rename_Engines.md), [Kotlin options](kotlin_Options.md)).

## What the decision rested on

A rename tool is only as good as the weakest answer it accepts without checking. Three questions decided each choice, and every candidate was asked all three on real code:

1. **Complete?** Does it edit every place that refers to the symbol, including other files, re-exports, keyword arguments, and tests?
2. **Faithful?** Does it leave every other name alone (parameters with the same spelling, a local in another scope)?
3. **Safe?** Does it refuse a new name that collides with something in scope, or would refac have to notice?

The third answer was "no" for every server of every language that was run. That is why the engine has its own proof (references before, references of the renamed text in memory after, the two sets must match) and why the choice of server is mostly about the first two questions and about how the server announces that it is ready.

## Candidates by language

| Language | Candidates | Chosen | Reason |
| :--- | :--- | :--- | :--- |
| Go | gopls v0.23.0 | gopls | The only full Go language server. It renames interface families and test variants by itself. |
| Rust | rust-analyzer 1.99.0 (the rustup component), the embedded `ra_ap_*` crates already used for `move-module` | rust-analyzer as a server | Same engine as the crates, but through the protocol every other language uses, so the proof and the report are shared. The crates stay for module moves. |
| Python | basedpyright 1.40.2, pyright 1.1.414, Pyrefly 1.3.2, ty 0.0.85, Rope 1.15.0 | basedpyright | Complete on eight of nine cases, and the only server that can list overrides, which closes the ninth. Details below. |
| Dart | the SDK's analysis server (`dart language-server`, Dart 3.13.5) | the analysis server | The only one. Needs one settle step, see below. |

## Python: five engines, one matrix

Python is typed only where the code says so, so every engine renames what it can prove and misses the rest. The study project has a package with `__init__.py` re-exports and an `__all__` list, a class with a subclass that overrides and calls `super()`, a property, a class attribute, a constant used through a module alias, a dataclass-like attribute passed by keyword, and a test module. Nine renames (C1 to C9) are compared with the places that have to change; five more (K1 to K5) are renames to a name that already exists and must be refused.

| Case | Server result that is required |
| :--- | :--- |
| C1 function `open_account` | the definition, the `__init__.py` import, the string in `__all__`, `app.py`, the test |
| C2 attribute `balance` | the attribute in the class and its uses in other modules, but not the constructor parameter or the keyword of the same spelling |
| C3 method `deposit` | the base method, the subclass override, the `super().deposit` call, calls through both classes |
| C4 parameter `amount` | inside its own function only |
| C5 class `Account` | definition, import, `__init__` re-export, annotations, tests; an `import ... as` alias keeps its spelling |
| C6 constant `MAX` | definition, the use in the same module, the use through `util.MAX` |
| C7 property `label` | definition and the attribute reads in `app.py` |
| C8 class attribute `interest_rate` | definition and the instance use |
| C9 local `total` in `clamp` | inside the function only |

Result, per engine (OK = exactly the required places; DIFF = places missed; nothing in any engine edited a place it should not have):

| Case | basedpyright | pyright | Pyrefly | ty | Rope |
| :--- | :--- | :--- | :--- | :--- | :--- |
| C1 | OK | OK | missed `pkg/__init__.py:3` (`__all__`) | missed `pkg/__init__.py:3` | missed `pkg/__init__.py:3` |
| C2 | OK | OK | missed `app.py:16`, `app.py:18`, `tests/test_core.py:8` | same three | OK |
| C3 | missed `pkg/core.py:29` (override), `app.py:15` | same | missed `app.py:13`, `tests/test_core.py:7` | missed 5 places | OK (with `unsure=True, in_hierarchy=True`) |
| C4 | OK | OK | OK | OK | OK |
| C5 | OK | OK | missed `pkg/__init__.py:3` | same | missed `pkg/__init__.py:3` |
| C6 | OK | OK | OK | OK | OK |
| C7 | OK | OK | missed `app.py:16` | same | OK |
| C8 | OK | OK | OK | OK | OK |
| C9 | OK | OK | OK | OK | OK |
| K1 to K5 (clashes) | all five accepted | all five accepted | all five accepted | all five accepted | all five accepted |

Findings:

- **Pyrefly and ty are not complete across files.** They resolve the attribute or the re-export only when the receiver's type is inferable in the file that uses it, and miss the rest without an error: five misses of nine for Pyrefly, five for ty, and the misses are always in files other than the one the rename starts from. An engine that returns partial edits and no warning is worse than none, so neither is used.
- **pyright and basedpyright are identical** on rename (basedpyright is a fork that adds options and a bundled Node.js). Their one miss is the override family: renaming `Account.deposit` edits the base method, calls through `Account`, and nothing else.
- **Rope is not a language server** (a library, driven by `refac`'s Python helper for moves). It handles the family when asked with `unsure=True, in_hierarchy=True`, but it misses the string in `__all__`, which makes the import fail at run time, and it accepts every clash.
- **No engine detects a clash.** `total` renamed to an existing local, a method renamed to an existing member, a function renamed to an existing module name, a parameter renamed to a global the body reads, an attribute renamed to an existing attribute: five of five accepted by all five, each producing a program that runs and does something else. This is the case the proof exists for: after the rename the references of the renamed declaration no longer equal the old ones carried through the edits.

### Closing the override gap

Overrides can be found with `textDocument/implementation`, which basedpyright answers and pyright does not (pyright announces no `implementationProvider`). On a class method, implementation lists the overrides in subclasses; on a class it lists the subclasses. `src/drivers/lsp/rename/plan/family.rs` asks at the renamed symbol, keeps the answers spelled like the symbol (a class's subclasses are not renamed), renames each override with the same new name, and merges the edits, dropping identical ones. Each override becomes a related group, so the proof checks it. On C3 the engine run on `app.py` line 13 edits six places in three files, which is the required set. Starting from an override renames only that override and its subclasses, because there is no request that finds the base method from below; the leftover note lists the base. basedpyright is the only server refac accepts for Python for this reason, and a server that does not announce `implementationProvider` is refused at start ("does not offer implementationProvider"), by the same `missing_capabilities` check that runs in `refac doctor`.

Untyped receivers (`thing.area()` on a parameter without an annotation) are not linked to the symbol by any engine, because the type is unknown. The leftover scan reports them by line.

## Go: gopls answers a rename with only part of the edits under load

gopls renames interface families, test variants in the same and external test packages, and doc comments that start with the symbol name, and it refuses collisions in scope and method renames that would stop a type implementing its interface. Findings that shaped the integration:

- **Readiness is a progress notification.** Only a client that announces `window.workDoneProgress` gets it; the "Setting up workspace" progress ends when the packages are loaded. No sleep is involved: the wait ends on that notification.
- **Partial answers under load.** With the machine busy, about one rename in four came back with only the edits for the named symbol: the implementing methods in other packages and the test variant were missing, while `references` was complete. Asking again after putting the documents back removes it, because the answer is a function of how far the background load has got, not of the request. The engine's completeness check (every listed reference must be edited) detects it as `Unfaithful`, and Go allows four attempts. Without the retry the symptom was a verification failure that vanished on a quiet machine. With it, the nine real-server tests passed in 16 of 16 runs under load.
- **Package rename is file operations.** Renaming the `package` clause returns a `rename` of the directory and deletes. The engine refuses any answer that contains file operations unless the language opts in, and points to `refac move`.
- **Edits on comment lines.** The doc comment of a renamed function is edited. Those edits lie outside every reference, so the engine accepts an edit on a `//` line that replaces exactly the old name and nothing else (`comments.rs`).

## Rust: rust-analyzer is right about code, silent about macros and files

- **Toolchain from the directory.** `~/.cargo/bin/rust-analyzer` is a rustup stand-in. It exists whether or not the component is installed, and fails with "Unknown binary 'rust-analyzer'" when it is not. The toolchain is chosen from the working directory of the process, so a project with a `rust-toolchain.toml` needs the server started in the project. The locator runs the version command before accepting a candidate and starts the server with the project as its working directory.
- **Readiness** is `experimental/serverStatus` with `quiescent: true`, sent to a client that announces `serverStatusNotification`.
- **`macro_rules!` bodies are not renamed.** A method called inside a macro definition keeps the old name, and the program stops compiling. The server gives no sign. The engine reads the project's `macro_rules!` definitions (`languages/rust/`), and the report starts with an `ATTENTION` note naming the lines. The test edits the macro by hand and then runs `cargo check` to prove the rest was right.
- **Module names are file moves.** Renaming a `mod` answers with a file rename; refused with a pointer to `refac move-module`.
- **Shorthand initialisers tie a field to a local.** `Rect { width, height }` renames the parameter together with the field; the edit lies outside the field's references. The engine accepts it because the server says that spot is itself a reference of another symbol, and checks that symbol as a related group.
- **Cost.** About 4.6 seconds and 637 MB for the fixture; see the table at the end.

## Dart: the analysis server answers from files it has not analysed yet

The Dart server renames override hierarchies, `export ... show` lists, named arguments, field formals, and `[Name]` links in doc comments by itself. The one trouble was in the proof, not in the rename.

- **Readiness**: `$/analyzerStatus` with `isAnalyzing: false`, sent when the client does not announce work-done progress.
- **The race.** After the proof shows the server the renamed text, it takes `didChange` in without analysing it, and the references it then answers are made from the files it has analysed so far. Five or six of the nine scenarios failed verification on every run, always on usages in files that were not open before ("usages that no longer refer to the symbol").
- **The status cannot be the signal.** Waiting for `$/analyzerStatus` after the change did not work: a status from before the change is still buffered (a stale `isAnalyzing: false` ends the wait at once), and the status for the change arrives after the answer it concerns.
- **A barrier request works.** `textDocument/semanticTokens/full` on a changed document is answered only when that document is fully resolved. Asking it for every changed document before the references makes the answers deterministic. The engine's `settle` step does that for servers that declare an `analysis_barrier`; the others do nothing. After this, nine of nine passed on repeated runs and under load.
- **`.dart_tool/package_config.json` is required.** Without it `package:` imports do not resolve and the rename silently misses every file that imports the symbol, so refac stops before any server starts and says "Run `dart pub get`".

## Real projects, not only fixtures

The fixtures are small, so each language was also run on real code (2026-10-07), with the project compiled or analysed before and after:

| Language | Project | Rename | Cost | Result |
| :--- | :--- | :--- | :--- | :--- |
| Go | `golang.org/x/tools` v0.51.0 copy, 1,286 `.go` files | `astutil.PathEnclosingInterval` (a function with nine callers in other packages and test files) | 8.4 s, 935 MB | 13 edits in 7 files; `go build ./...` and `go vet` of the changed packages pass. The 25 lines still spelling the name are comments, test messages, and `loader.Program.PathEnclosingInterval`, a different method |
| Rust | this repository | `file_uri` (a function used in six files, plus an unrelated function of the same name in `lsp/client.rs`) | 32.7 s, 1.9 GB | 12 edits in 6 files; `cargo check --lib --tests` passes; the three lines in `lsp/client.rs` were listed as untouched |
| Python | Rope 1.15.0, 97 files | class `RefactoringError` | 2.1 s, 331 MB | 61 edits in 14 files, nothing left; basedpyright reports the same 616 errors and 20,912 warnings before and after, and the modules import |
| Python | Rope 1.15.0 | method `get_kind`, defined in 12 classes of a mostly unannotated code base | 2.7 s, 345 MB | 5 edits; 39 lines in 17 files are listed as untouched, because their receivers have no type. The limit is real and the report says so |
| Dart | `collection` 1.19.1 from pub.dev, 29 library files and its tests | class `HeapPriorityQueue` | 0.9 s, 170 MB | 19 edits in 2 files; `dart analyze` reports the same 5 issues and the 42 tests of that file pass |
| Dart | the same package | `QueueList`, `ListEquality` | 3 to 6 s | refused, nothing written: other packages in the pub cache (`async`, `analyzer`) use these classes and no server edits dependencies |

The last row is why the proof now treats references outside the project folder as their own failure. The first version reported "leaves 58 of the 104 places unchanged" and named a file in `~/.pub-cache`, which is correct and not obvious. The message now says the symbol is used by other packages, that the server never edits them, and that a symbol other packages use is part of the project's public interface. It is not retried (gopls gets four tries for a different problem; a file outside the project can never be edited).

The shapes of project were checked too: a project below a hidden `.`-folder, a symlinked project path (Go, Python, Rust, Dart), a Cargo workspace of two crates, a Go `go.work` with two modules, and a Python `src/` layout with `pyproject.toml`. All behave. Non-ASCII text before a symbol on the same line (an emoji is two UTF-16 units) and CRLF files are covered by `tests/rename/encoding.rs`, which fails on all four languages when the UTF-16 offset code is deliberately broken.

## Server lookup, and what a missing server must say

Not an engine question, but it decided how the servers are found: all four are installed by someone else, and an agent that meets "No such file" has nothing to act on. The locator ([Language servers](../Setup/language_Servers.md)) tries the environment variable (a wrong value is final), then `PATH`, then the folders installers use, requires each candidate to run its version command, records every place it looked, and prints that record with `Run refac doctor <language>`. The case that made the version check mandatory is the rustup stand-in above: a file that exists, is executable, and cannot serve.

## Cost of one rename, measured

`rename --dry-run` on the test fixtures, debug build, a quiet machine, server started and stopped inside the command:

| Language | Wall time | Largest process |
| :--- | :--- | :--- |
| Go (gopls) | 1.7 s | 165 MB |
| Rust (rust-analyzer) | 4.6 s | 637 MB |
| Python (basedpyright) | 1.3 s | 158 MB |
| Dart (analysis server) | 0.4 s | 124 MB |
| Kotlin | about 38 s | about 1.6 GiB, plus the Gradle daemon |

Nothing is left running afterwards; that was a design rule, because an idle server holds hundreds of megabytes and an agent runs a rename a few times per task.

## Keep the server alive between commands? No, batch instead

The question: several renames in a row each pay the start (above), so should an option keep the server alive for a while? Weighed and rejected for now:

- **Stale view.** The in-memory proof is only as good as the server's picture of the files. A resident server would hold documents that an editor, `git checkout`, a formatter, or another refac command has since changed; every request would need every changed file revalidated and re-sent, and a mistake there turns the proof into a false assurance. Observed cases that already needed care with a fresh server: the Dart server answers from files it has not analysed yet, gopls answers a rename with part of the edits under load, and the Kotlin server's file watcher is asynchronous.
- **Memory.** Idle servers hold 0.1-2 GB each (table above); per project and language, for as long as the time-to-live runs. The design rule is that nothing stays resident.
- **Process management.** A socket or pid file, a time-to-live, cleanup after a crash or a killed agent, one server per project and language, and an environment where the process may not outlive the command at all (the remote sandbox this was built in).
- **What it would save.** Only the start. Most starts are small: Go 1.7 s, Python 1.3 s, Dart 0.4 s, and the fixtures' Rust 4.6 s. The expensive ones are Kotlin (about 38 s) and a large Rust project (33 s, 1.9 GB).

What was built instead is `refac rename --batch`: all renames of one task in one call, one server start, one stop, all or nothing. Measured on the Rust fixture, four renames: 18.8 s as four commands, 4.6 s as one batch, identical files. The batch keeps the proof honest (each rename is planned against the files as the previous one wrote them, the server is told what changed) and nothing is left behind to warn about; the skill teaches agents to put the renames of one project into one batch.

Revisit for Kotlin only, and only opt-in: a time-to-live, revalidation of every file that changed on disk since the last request, and an explicit message in the output that a server of about 1.6 GiB stays resident until a stated time.

## Revisit when

- Pyrefly or ty resolve attributes and re-exports across files (rerun the nine cases from a project that follows C1 to C9) or pyright announces `implementationProvider`.
- A server starts to detect name clashes. The proof would still run, but the clash cases could then be refused earlier, with the server's own message.
- gopls answers a rename completely under load; the four attempts for Go could then drop to one.
- rust-analyzer renames inside `macro_rules!`; the `ATTENTION` note and the macro scan could go.
