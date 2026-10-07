# TypeScript Rename Engines

Evidence behind the choice of engine for `refac rename`. It records which tools were tried, what each did on the same test cases, what scale did to them, and which silent failures turned up. Read it before changing the engine or adding another language's rename. The command itself is described in [Symbol Rename](../Features/TypeScript/symbol_Rename.md).

All measurements are from 2026-10-07 on one Linux machine. They are workload-specific local numbers, not benchmarks.

## Method

Every engine received the same rename requests, and every result was converted to the same edit format, applied to a copy of the project, diffed, and typechecked. The small fixture has 9 files and 16 cases: exported const, import-site start, shadowing local, shadowing parameter, aliased function, class, property (declaration and usage start), method, interface, enum member, default function, local with `export { }`, JSX component, standard-library symbol, and a deliberate name collision. The large project is a copy of the TanStack Router packages with one tsconfig and workspace path mappings: 2,103 TS/TSX/JS files.

## Candidates

| Engine | Rename | Result |
|---|---|---|
| TypeScript 7.0.2 native (`tsc --lsp --stdio`) | yes | 15 of 15 edit cases correct, byte-identical to TypeScript 6. 5 to 7 seconds and 250 to 400 MiB of engine memory on the large project. Follows project references. **Chosen.** |
| TypeScript 6.0.2 LanguageService (JS API) | yes | 15 of 15 correct. 10 to 24 seconds and 460 to 640 MiB on the large project. One tsconfig, no project references in this harness. Works with legacy configs. This is also what ts-morph wraps. |
| typescript-language-server 6.0.1 | yes | Wraps the TypeScript 6 server. In the test harness it answered with the declaring file's edits only and took 5 to 9 seconds per request on the tiny fixture. Not pursued: it is the same engine as TypeScript 6 behind a slower wrapper. |
| Biome 2.5.15 language server | no | Reports no providers; `textDocument/prepareRename` is "method not found". |
| oxlint 1.87 language server | no | Offers code actions only. |
| Oxc parser 0.153 (JavaScript binding) | no | Exposes `parse` and a visitor, no scope or symbol API. Rust `oxc_semantic` has scopes but no cross-file or type-aware rename, so members would still need a type checker. |
| ts-morph 28 | via LanguageService | Not retested. The earlier ts-morph move path reached about 21.5 GiB on a large task, see [TypeScript](../Features/TypeScript/linker_TypeScript.md). |

TypeScript 7 no longer exports the classic JavaScript API: the package root exports only a version file, plus `unstable/*` client entry points. File moves therefore stay on TypeScript 6 in the Bun helper, and rename runs the TypeScript 7 binary through the locked `typescript-native` alias dependency.

## Findings that shaped the design

1. **Neither engine detects name clashes or shadowing.** Renaming `total` to an existing `computeSum` produced duplicate declarations that still renamed 9 sites. Renaming a local `inner` to a visible `outer` produced valid code that silently changes meaning: `inner + outer` becomes `outer + outer`. Both compile or nearly do, so a post-rename typecheck is not enough. The references round trip in [Symbol Rename](../Features/TypeScript/symbol_Rename.md) catches both. Counting references spelled with the new name is deliberate: counting all references also counts `renamed as total` alias nodes and flagged 2 of 8 legitimate renames.
2. **TypeScript 7 silently misses files when the tsconfig has `baseUrl`.** The rename returned 4 edits instead of 9 and reported no error, because the engine ignores the removed option and cannot resolve the import. Listing files with `-p . --listFilesOnly` exits with code 1 and prints `TS5102` for `baseUrl` and `TS5108` for `moduleResolution=node10`, while exiting 0 for projects that merely have type errors. That listing is the pre-flight guard, so no list of removed options has to be maintained.
3. **Project discovery decides completeness.** From a package root the engine uses the nearest tsconfig. Usages in another project are found only when the engine can reach that project, for example through a solution-style root tsconfig with `references`, in which case the edits land outside the project path and Refac refuses. Usages in unreachable projects are invisible, which is the same coverage rule file moves have.
4. **Protocol details.** The server requires `rootUri` in `initialize`. It negotiates `utf-8` positions when offered, so every offset is a byte offset. It sends `client/registerCapability` as a request that must be answered. It refuses `node_modules` and standard-library symbols with an explicit message. LSP file URIs are percent-encoded (`@` arrives as `%40`) and must be decoded.
5. **TypeScript's replacement text keeps public names.** Renames come back as `{ total: grandTotal }`, `grandTotal as total`, or `grandTotal: total` rather than plain identifiers, so verification has to locate the new name inside each replacement. Unknown replacement shapes make the rename fail rather than guess.

## Scale

| Case (2,103 files, one tsconfig) | Edits | TypeScript 7 native | TypeScript 6 LanguageService |
|---|---|---|---|
| `notFound` function in router-core | 4 in 3 files | 1.6 s, 246 MiB | 9.7 s, 465 MiB |
| `invalidate` member, used across packages | 218 in 31 files | 5.0 to 7.0 s, 394 MiB | 22 to 24 s, 640 MiB |
| Full `tsc --noEmit` of the same project | none | 5.0 s, 274 MiB | 39.9 s, 429 MiB |

Both engines returned the identical 218 edits. The finished command, including pre-flight, verification, and a debug build, took about 13 seconds with the engine near 535 MiB at peak.
