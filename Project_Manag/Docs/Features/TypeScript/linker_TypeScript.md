# TypeScript / JavaScript

TypeScript and JavaScript have two operations with two engines. File and directory **moves** use Oxc to parse each source file, TypeScript 6 to resolve code imports, and Oxc Resolver for asset requests. Precise literal edits preserve other source bytes. The Bun helper creates no TypeScript Program, type checker, language service, or ts-morph project. **Symbol renames** need the type checker, so they use the TypeScript 7 native language server from Rust; see [Symbol rename](#symbol-rename).

## Required tooling and scope

Bun runs `scripts/ts_refactor.ts`. Missing dependencies are installed from `scripts/bun.lock` with `bun install --frozen-lockfile`.

Point `--project-path` at the package owning `tsconfig.json`. TypeScript reads config inheritance, `files`, `include`, `exclude`, aliases, and resolution options. Every configured source is scanned, plus explicitly moved sources; dependencies are resolved on demand without loading their ASTs. Without a tsconfig, the helper uses TypeScript's default file discovery with `allowJs`, ESNext modules, and bundler resolution.

## Symbol rename

`refac rename` renames one variable, function, class, interface, enum, or member and every reference to it. The TypeScript 7 native binary (`tsc --lsp --stdio`, installed from the locked `typescript-native` alias in `scripts/package.json`) finds the references. Refac plans all edits first, proves in memory that the new name neither clashes with nor shadows another symbol, and only then writes, with rollback. Details, flags, limits, and hard failures are in [Symbol Rename](symbol_Rename.md). Why this engine was chosen over TypeScript 6, ts-morph, Biome, and Oxc is in [TypeScript rename engines](../../Investigation/typescript_Rename_Engines.md).

## Implementation owners

- [CLI entry](../../../../scripts/ts_refactor.ts): argument parsing and error reporting.
- [TypeScript modules](../../../../scripts/TypeScript/): `project.ts` reads config; `imports.ts` collects literal spans; `resolver.ts` resolves and spells paths; `moves.ts` validates requests; `plan.ts` plans and verifies the batch; `apply.ts` owns filesystem changes and rollback.
- [Rename driver](../../../../src/drivers/typescript/rename.rs): request validation, limits, and orchestration. Its folder holds one file per job: `engine.rs` finds the native binary and runs the config pre-flight, `session.rs` speaks the language-server protocol, `locate.rs` finds candidate occurrences, `plan.rs` resolves them to one symbol and collects edits, `verify.rs` runs the in-memory references check, `edits.rs` converts and applies edits, and `apply.rs` writes with rollback.
- [Behavior tests](../../../../scripts/Tests/TypeScript/): syntax, paths, batches, safety, and a 3,005-file stress fixture.
- [Rename tests](../../../../tests/typescript/rename.rs): 19 CLI tests on `tests/fixtures/typescript/rename_project`, including the clash, shadow-capture, UTF-8, BOM, and config-rejection cases.
- [CLI stress test](../../../../tests/typescript/large_project.rs): five dependent moves and 3,000 callers under a 1 GiB RSS budget.

## Reference updates

Supported forms include imports, re-exports, side-effect imports, literal dynamic imports, CommonJS `require`, TypeScript import-equals, import types, and resolved module augmentations. No-substitution template literals are supported. Node ESM/CommonJS conditions and explicit `resolution-mode` attributes use TypeScript's resolver. Asset imports preserve query and fragment suffixes.

Aliases in `compilerOptions.paths` retain their spelling when the destination fits the same alias. Fixed aliases or moves outside an alias become explicit relative imports; tsconfig mappings themselves are not edited. Resolution is checked after all moves against each previously resolved target in affected files. A normal apply or verification error restores edited bytes and moved paths; rollback failure is reported explicitly.

## Dry run (`move --dry-run`)

`scripts/ts_refactor.ts` builds the whole plan (every moved file, every rewritten specifier) before it writes anything; with `--dry-run` it prints that plan as one JSON line (`moves`, `files` with the number of rewritten specifiers, and `unchecked`) and stops. The check that each rewritten specifier resolves from the new place also runs in a dry run: `scripts/TypeScript/virtual.ts` (`afterMoves`) gives the resolver a model of the files as they will be after the moves (a moved file exists at its target and no longer at its source), so a move whose new place another module answers (`src/old.ts` to `src/lib/index.ts` next to a `src/lib.ts`) is refused in the dry run with the message of the real move. The asset resolver reads the real disk, so an asset import is checkable only when it is written relative to the importer; a rewritten import of an asset through an alias is counted in `unchecked` and named in a note, and the real move checks it.

## Key limits

- **Coverage:** callers must be in the owning tsconfig. Project references are rejected until cross-project planning is supported.
- **Unsupported ambiguity:** malformed source/config, symlink move paths, overlapping requests, locally rebound `require` calls, and unresolved outgoing relative imports fail explicitly.
- **Manual audit:** computed module paths, arbitrary strings, JSDoc/triple-slash comments, framework path conventions, and package/config metadata are not rewritten. Search for old paths and run the target project's typecheck/build.
- **Batch size:** at most 30 contained TypeScript/JavaScript source files, counted before mutation.
- **Process limits:** the Rust supervisor terminates and reaps the helper after 5 minutes or sampled RSS above 4 GiB. RSS is sampled every 100 ms, so brief overshoot is possible. `REFAC_TYPESCRIPT_MAX_RSS_MB` sets a positive integer MiB threshold. Cancellation also terminates the child. Process termination or a crash can interrupt rollback: inspect the working tree before retrying.

## Performance evidence

On 2026-09-23, a disposable copy of Shadi Intake replayed the five Sentry moves across 3,150 configured sources in 1.1–1.8 seconds per batch, with peak helper RSS about 212 MiB. The reverse/forward round trip was byte-identical and the copied project passed typechecking. These are workload-specific local measurements.

The old ts-morph move path reached about 21.5 GiB in the reported task. Upstream [issue 1613](https://github.com/dsherret/ts-morph/issues/1613) and [issue 953](https://github.com/dsherret/ts-morph/issues/953) track slow move operations. Oxc parser spans matched TypeScript's parser for all 15,058 module references in the comparison. TypeScript's resolver is retained because general Oxc resolver options did not establish parity for every code-module resolution mode.
