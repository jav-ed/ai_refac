# Astro Options

Whether `refac` can move files and rename symbols in Astro projects, what it does today, and which tools could give it that. Astro projects mix `.astro` components, plain TypeScript, `.mdx` pages, CSS and assets, and a move or rename in one of them touches the others. Everything below was run on 2026-10-24.

**Status: implemented (2026-10-24, in the working tree, not yet released).** The investigation below is what `refac` 0.1.4 did and why the direction was chosen; the section [Result](#result) has what the implementation does now, re-run on the same fixture. The behaviour is documented in [Astro and MDX](../Features/TypeScript/astro_And_Mdx.md).

`refac` 0.1.4 had no Astro support and, in four of eight cases, reported success while leaving the project broken. The TypeScript 7.1 native server plus the Astro content mapper handles `.ts` and `.astro` moves and renames (proven on a fixture and on a real project) but is slower than the existing Oxc engine and needs a tsconfig without `baseUrl`. Moves therefore go through the Oxc engine, renames through the 7.1 server; MDX and non-TypeScript references are refac's own work. Details and order at the end.

## What refac 0.1.4 does today

Build `fixture` (Astro 7.4.2, `@astrojs/mdx` 8.1.1, `astro check` 0 errors, `astro build` 3 pages), copy it per case, run `refac`, then run `astro check` and `astro build` on the copy. Exit code and message are what an agent sees.

| Case | refac says | Project afterwards |
|---|---|---|
| S1 move `lib/format.ts`, imported by `.ts`, `.astro` and `.mdx` | exit 0, "successfully refactored", "verified 3 affected source files" | the two `.ts` importers fixed; 3 `.astro` and 1 `.mdx` importer left on the old path; check 3 errors, build fails |
| S2 move a `.astro` component | exit 1, skipped (unsupported extension) | unchanged |
| S3a move a folder that holds only `.astro` files | exit 1, directory moves not supported for this content | unchanged |
| S3b move a folder with `Hero.astro` and `hero.svg` | exit 0, "successfully refactored" as a Markdown move | folder moved, the `.astro` importer left broken |
| S4 rename a TypeScript export used in `.astro` and `.mdx` | exit 0, 3 edits in 2 files | `.astro` and `.mdx` users keep the old name; check 3 errors, build fails |
| S5 rename a symbol declared inside a `.astro` file | exit 1, lists the supported extensions | unchanged |
| S6 move `logo.svg` imported by `.astro` and linked from Markdown | exit 0, "successfully refactored" as a Markdown move | the Markdown link fixed; `import logo from "../assets/logo.svg"` and a CSS `url()` left; build fails |
| S7 move a `.css` file imported by `.astro` and by another `.css` | exit 1, skipped | unchanged |

Causes. The TypeScript driver reads only `ts tsx js jsx mts cts mjs cjs` (`src/logic/grouping/route.rs`), the native server has no `.astro` file in its program, and the Markdown driver fixes `.mdx` links but not `.mdx` imports. `astro` is not in `CODE_EXTENSIONS`, so a folder with an image and a `.astro` file counts as a "document folder" (S3b). The refusals (exit 1) are safe; the four exit-0 cases are not, because an agent trusts the message and moves on.

## Candidates

**Astro language server** (`@astrojs/language-server` 2.17.3, built on Volar). Driven over LSP with `lsp_probe.mjs`: `textDocument/rename` renames across `.ts` and `.astro` files and a symbol declared in `.astro` frontmatter; `workspace/willRenameFiles` fixes importers in `.astro` and `.ts` files. It misses `.mdx`, the `<script>` import of a moved component, assets and CSS. It crashes (`TypeError` in `volar-service-typescript`, `getConfigTitle` gets no document, the process exits) on a move whose old path is a folder, an image or a CSS file. It needs the Node and Volar stack and a TypeScript 5 or 6 JavaScript API.

**TypeScript 7.1 with `@astrojs/ts-content-mapper`** (`typescript` 7.1.0-beta, mapper 0.2.0). Content mappers are a TypeScript 7.1 feature (microsoft/typescript-go#4712 merged 2026-08-19, microsoft/TypeScript#63936 merged 2026-08-21): a package turns a `.astro` file into virtual TypeScript with a source map, and the native server answers for the original file. `astro check` itself refuses TypeScript 7 and points to this mapper. It is the same engine `refac` already starts for TypeScript rename (`tsc --lsp --stdio`). Setup: `contentMappers` in `tsconfig.json`, `initializationOptions.runExternalCode = true` (the mapper is code from `node_modules`, so the server needs an explicit opt-in), then the request `custom/setContentMapperContributions` with the extensions. `tsc --noEmit --runExternalCode` reports a deliberate type error in `.astro` frontmatter and at its use in the template.

**Looked at, not run:** `@astrojs/ts-plugin` (a tsserver plugin that runs only in editors; not tried with the native server), `ts-refactor-mcp` and `codemode-lsp` (TypeScript file moves and refactors through MCP, no Astro in their descriptions), `astro-rename` (a PostCSS integration, unrelated). GitHub searches for "astro refactor", "astro rename files" and "move file update imports mcp" found no tool that moves Astro files; the real building blocks are the two above.

## Proof: TypeScript 7.1 plus the mapper, edits applied to copies

`run_lsp_scenarios.sh` asks the server for the edits, writes them into a copy of the fixture, moves the file or folder, then runs `tsc --noEmit --runExternalCode` and `astro build`.

| Case | tsc | astro build | What is left |
|---|---|---|---|
| L1 move `lib/format.ts` | pass | fails | only `second.mdx` (`import ... from "../../lib/format"`) |
| L2 move `Card.astro` | pass | fails | `second.mdx` import, `import logo from "../assets/logo.svg"` and the CSS `url("../assets/logo.svg")` inside the moved file |
| L3 move folder `components/hero` (`.astro` plus image) | pass | pass | nothing |
| L4 rename `formatDate` | pass | fails | `second.mdx` (the import and one use) |
| L5 rename `when`, declared in `Card.astro` frontmatter | pass | pass | nothing |

It also fixed what the Astro server missed: the import inside the `<script>` block of the moved `Card.astro` (`../lib/track`), and the folder move did not crash. Every `.astro` and `.ts` importer was updated with correct positions in the `.astro` files.

## What no engine covers

1. `.mdx`: `import` and `export` lines, and uses of imported names in the body, both for moves and for renames.
2. References that TypeScript does not resolve: an asset import (`import logo from "../assets/logo.svg"`), CSS `url()` and `@import`, `<img src>` in a template, in a moved `.astro` file and in the files that point at a moved asset or stylesheet (S6 and S7).
3. String globs such as `import.meta.glob("../components/*.astro")`: nobody updates these.

## A real project: speed, tsconfig and who imports what

The same two engines on a copy of a production Astro site (290 `.astro`, 188 `.ts`, 122 `.mdx` files, 12 path aliases that rely on `baseUrl`). Only the copy was changed.

| Measure | Result |
|---|---|
| Who imports `route_Slugs.ts` | 60 `.astro`, 9 `.ts` (plus relative imports in siblings), 1 `.js` |
| Who imports the component `Mermaid_Diagram.astro` | 44 `.mdx`, 1 `.ts`: MDX is the main importer, and no engine reads it |
| Current Oxc engine, move `route_Slugs.ts` (dry run, 5 runs, original tsconfig with `baseUrl`) | 324 to 340 ms, 18 edits in TypeScript files, the 60 `.astro` importers untouched |
| TypeScript 7.1 server, same move (3 runs) | initialize 27 ms, mapper setup 1 ms, move request 553 to 565 ms; edits in 60 `.astro`, 17 `.ts` and 1 `.js` file |
| TypeScript 7.1 `tsc --noEmit` through the mapper | all 290 `.astro` files in the program, 3 errors (virtual modules of a third-party integration) |
| The project's own `tsconfig.json` on TypeScript 7.1 | **fails outright**: `TS5102 Option 'baseUrl' has been removed` and 12 times `TS5090 Non-relative paths are not allowed`; it ran only after the copy's tsconfig was migrated (no `baseUrl`, `./` before every alias target) |

Small fixture, for comparison: both engines took about 0.2 s. So the language server costs about 1.7 times the Oxc engine on a real project, and it needs three things from the user's project that the Oxc engine does not: a tsconfig without `baseUrl`, the mapper package, and consent to run its code.

## Rust where it exists

`refac` runs its TypeScript moves on Oxc (a Rust parser and resolver) through Bun, with the npm packages `oxc-parser` and `oxc-resolver`; the engine is about 490 lines in `scripts/TypeScript/`. `oxc_parser` 0.155.0 and `oxc_resolver` 11.24.3 are on crates.io, so the same engine could be linked into the Rust binary and lose Bun for moves (not measured). Astro's own compiler is Rust (`withastro/compiler-rs`, crates `astro2tsx` and `astro_codegen`, built on a forked Oxc taken by git revision), but it is not on crates.io, so it would be a git dependency. For symbol rename no Rust engine exists: the TypeScript 7 native server (Go) is the only one that sees `.astro`.

## Recommendation and decisions

Decided by the owner on 2026-10-24: a pre-release TypeScript 7.1 is acceptable, the fastest engine that works wins, and the docs say to check now and then for a newer version.

1. **Stop the silent success first.** Add `astro`, `vue` and `svelte` to `CODE_EXTENSIONS` (turns S3b into a refusal), and make a TypeScript move or rename fail with the list of files when `.astro`, `.vue`, `.svelte` or `.mdx` files outside the engine's reach still reference the moved path or the renamed name. Small, no new dependency, errors stay hard.
2. **Moves: extend the Oxc engine, not the language server.** It is faster on the real project (about 330 ms against about 580 ms before the `.astro` work is added), it reads `baseUrl` and the aliases of existing projects, and it needs no mapper, no consent flag and no pre-release. `collectImports` already returns exact offsets, so the work is to hand it the script parts of the file: the frontmatter and each `<script>` of an `.astro` file, and the `import` and `export` lines of an `.mdx` file (the 44 MDX importers above). Asset, CSS and `<img src>` references inside `.astro` are the same kind of work and stay refac's own, in the style of the Markdown driver. Porting the whole engine to the Rust crates is a separate, later step that would also drop Bun.
3. **Renames: TypeScript 7.1 beta plus the mapper**, because nothing else can rename inside `.astro`. Opt in explicitly (a flag or environment variable, never a default) because the mapper is code from `node_modules`; a project without the mapper in its `tsconfig.json`, or with `baseUrl`, gets a hard error with the exact lines to change. Renaming a name inside `.mdx` has no engine.
4. **Docs:** one short note that the TypeScript 7.1 engine and the mapper are young and the reader should check from time to time for a newer version.
5. **Report upstream:** the Astro language server crash on a folder, image or CSS move.

## Rerunning

The kit lives in `Trials/Astro/` (gitignored, local to this machine): `fixture/` and `fixture_ts71/` (the project, with TypeScript 6 and 7.1 respectively), `run_scenarios.sh [refac-binary]` (S1 to S7), `lsp_probe.mjs` (asks a language server for edits, `apply` mode writes them), `run_lsp_scenarios.sh` (L1 to L5). When implementation starts, move the fixture and the scenarios into `tests/`.

## Result

Implemented as decided. `Trials/Astro/run_scenarios.sh` (the same fixture and scenarios as above, `astro check` and `astro build` after each) against the new build:

| Case | refac 0.1.4 | now |
|---|---|---|
| S1 move `lib/format.ts`, imported by `.ts`, `.astro`, `.mdx` | exit 0, 4 importers left broken, build fails | exit 0, `astro check` 0 errors, build passes, 0 stale references |
| S2 move a `.astro` component | skipped | pass, same checks |
| S3a move a folder of only `.astro` files | refused | pass |
| S3b move `components/hero` (`.astro` plus image) | exit 0 as a Markdown move, importer broken | pass (moves as code) |
| S4 rename a TypeScript export used in `.astro` and `.mdx` | exit 0, users keep the old name, build fails | exit 1, nothing written, the users are listed; with `--astro` (mapper) `.astro` users follow, `.mdx` users still stop it |
| S5 rename a symbol declared in `.astro` | refused | exit 1 without `--astro` (the message names it); renamed with `--astro` |
| S6 move `logo.svg` (imported, `url()` in `<style>`, Markdown link) | exit 0, import, `url()` left, build fails | exit 0, imports and `url()` and the link follow |
| S7 move a `.css` file (imported by `.astro`, `@import`ed by another `.css`) | skipped | importers and own `url()` follow |

What the implementation is, in the order of the plan:

1. **Safety.** `astro` is a code extension everywhere; a file the engine cannot parse stops the move with its name.
2. **Moves in the Oxc engine.** `.astro` is read with `@astrojs/compiler-rs` (frontmatter, `<script>` without attributes or with only `src`, `<style>`), `.mdx` by its `import`/`export` blocks, `.css` as a stylesheet. Pages and pictures that code imports stay with the Markdown driver; the engine runs first with `--external` and renames nothing. A first attempt that sent those files through the TypeScript group broke the Markdown driver's handling of dangling links, so it was replaced.
3. **Renames** on the same TypeScript 7.1 beta server `refac` already started, opt-in with `--astro` because the content mapper is code from `node_modules`. Measured on a copy of a real site (290 `.astro`, 188 `.ts`, 122 `.mdx`): Oxc move 324 to 340 ms against 585 ms through the server; the real site's tsconfig needs `baseUrl` migrated first (TS5102).
4. **Found by moving real files of the site and building** (a copy of the real site, `astro build` before and after): `@import "../Base/base_Main"` without `.css` was missed (Vite adds the extension), and the strings that name files and folders were not followed (`base: "./src/Content/…"` of 16 content collections, font files in a config object). Both are fixed. A probe project showed that Astro copies `<img src>`, `srcset`, `poster` and `<link href>` to the page as written, so they are URLs and stay; only a `<script src>` with no other attribute is bundled as an import. `import.meta.glob` is used in five of the owner's other projects, so its folder part follows moves. The largest find came from the build sweep: every one of the 122 `.mdx` pages of the real site names its pictures by relative path in the YAML frontmatter (202 values, `src:` and `image:`), which the Markdown driver does not treat as links, so moving a picture or a page broke the build. Those values follow moves now.
5. **Not done**, by choice: `public/` URLs, content-collection ids, the membership of a glob, Sass/Less files, `.mdx` rename, Vue and Svelte (the owner put them later).
6. **Upstream.** The Astro language server crash on folder, image and CSS moves (`TypeError` in `volar-service-typescript`) is still unreported.

Versions to look at now and then: `typescript-native` (7.1 beta, an exact version in `scripts/package.json`), `@astrojs/ts-content-mapper` (0.2.0), `@astrojs/compiler-rs` (0.5.2).
