# TypeScript, JavaScript, Astro and MDX

Read before the first TypeScript, JavaScript, Astro or MDX move or rename, and when one fails. Moves and renames use two separate engines: the move engine (the Oxc parser and TypeScript's resolver, run by Bun) and, for `rename`, the TypeScript 7 native language server.

## Setup

- `--project-path`: the package root whose `tsconfig.json` includes every caller. TypeScript 7 must accept the tsconfig (no `baseUrl`, no `moduleResolution: node10`). Without a `tsconfig.json` the callers are the files git lists (tracked, or untracked and not ignored), so a vendored or build folder in `.gitignore` is never scanned; outside a git work tree every file under the root is read, and if git is not installed the command stops and says so.
- Tool: `bun` ([bun.sh](https://bun.sh)), found in `PATH` and then in `~/.bun/bin/bun`. The Oxc parser, the resolver and the TypeScript 7 engine are installed on the first run with `bun install --frozen-lockfile`.
- `rename --astro` also needs `node` ([nodejs.org](https://nodejs.org)), `@astrojs/ts-content-mapper` installed in the project (`npm i -D @astrojs/ts-content-mapper`) and `contentMappers` in the tsconfig (see Astro and MDX below).
- `refac doctor typescript` checks the engine.

## Move: tsconfig coverage and limits

At most 30 source files per call (a folder counts the files below it), and only files the tsconfig includes are updated callers.

Point `--project-path` at the package containing the authoritative `tsconfig.json`. Its `include` or `files` configuration must cover all local TS/JS sources that participate in imports. External packages in `node_modules` do not need to be included.

Refac scans the complete tsconfig source set with Oxc and resolves code imports with TypeScript without constructing a type checker. Batches remain limited to 30 contained source files. Ordinary apply/verification failures roll back; a crash or forced termination can interrupt rollback.

The helper is terminated and reaped after 5 minutes or sampled RSS above 4 GiB (100 ms sampling). `REFAC_TYPESCRIPT_MAX_RSS_MB` changes the threshold in positive integer MiB. Inspect the working tree after a limit failure. One-file batches still scan every configured source; do not shrink coverage to hide callers.

### Reference-update gaps

Declared aliases are rewritten and verified. Project references, symlink moves, overlapping requests, and ambiguous locally bound `require` calls fail explicitly. Computed imports, arbitrary path strings, comments, and package/config metadata need manual audit. Search old paths and run the project build. `refac guide languages` and `refac move --help` list the supported forms and exact limits.

## Astro and MDX

The move engine reads `.astro` files (frontmatter, every `<script>` without attributes and the `src` of a `<script>` with only `src`, parsed by Astro's compiler) and `.mdx` files (top-level `import`/`export` blocks), so importers in them follow a moved module and a moved `.astro` or `.mdx` file has its own imports fixed. A file it cannot parse stops the move and is named; nothing is written. A page, picture or folder of them that code imports is moved by the Markdown driver, after the TypeScript engine has fixed the code (the engine is not started when no code uses the files).

A symbol rename is different: the language server reads `.astro` only through the project's content mapper. Without `--astro` a rename that `.astro` or `.mdx` files would not follow (they import from a file the rename edits and write the name) is refused with the files listed, and a tsconfig that lists `contentMappers` is refused with a hint. With `--astro` (or `REFAC_ASTRO=1`) the engine runs the mapper and `.astro` files follow, and a symbol declared in an `.astro` file can be renamed. Requirements (each one a hard error): Node.js, `@astrojs/ts-content-mapper` in the project, `"contentMappers": [{ "package": "@astrojs/ts-content-mapper", "extensions": [".astro"] }]` in `tsconfig.json`, and a tsconfig TypeScript 7 accepts (no `baseUrl`). `.mdx` files still stop a rename, with the files listed. A name that `aria-label`-style attributes share with a symbol makes the engine ask for `--line`.

`url()` and `@import` paths in `<style>` blocks and in plain `.css` files are rewritten like imports (a `.css` file can be moved; `@import "./base"` without `.css` stays without it). Strings in code that are exactly the root-relative path of a moved file or folder (`base: "./src/Content/blog"` of a content collection, a font in a config object) are rewritten, and so is the folder part of `import.meta.glob`/`Astro.glob` patterns that start with `./`, `../` or `/`, and the relative paths (`./`, `../`) in the frontmatter of `.mdx` and `.md` pages that name an existing file (`image: ../assets/hero.webp`, which a content collection reads relative to the page); the keys of a glob result change with its pattern, and a note says so. A file that moves into or out of the folder of a glob changes what the glob selects; no edit can prevent that, so a note names the file and the pattern. After the move another note lists the config and data files (`.json`, `.yaml`, `.toml`, `.html`, scripts) that still contain the old path of a moved file or folder; refac does not edit them. A project that calls `import.meta.glob` or `Astro.glob` in code starts the engine for every page or picture move so these notes are complete; a mention in Markdown prose, a fenced example or inline code is not a call and does not. Not rewritten, search for the old path afterwards and run the site's build: `<img src>`, `srcset`, `<link href>` and other template attributes (Astro copies them as URLs), `public/` URLs, content-collection ids, `tsconfig` `paths`; `.scss`/`.sass`/`.less` are neither moved nor read; `.vue` and `.svelte` importers are not read yet. A folder of stylesheets is moved by the TypeScript engine, so the paths in them and their importers follow.

## Rename a symbol

`refac rename` uses the TypeScript 7 native language server (installed from the locked `typescript-native` dependency by the first run's `bun install --frozen-lockfile`). It is a separate engine from file moves.

- Pick the symbol with `--file` and `--symbol`. If the name refers to several symbols in that file, the command lists the candidates; repeat it with `--line` (1-based) and, if one line has several, `--column` (1-based byte column).
- Nothing is written until the rename is planned and verified in memory. A new name that clashes with or shadows another symbol is refused, even when the result would still compile.
- TypeScript keeps public names: `{ total }` becomes `{ total: renamed }` and `export { total } from "./x"` becomes `export { renamed as total } from "./x"`. Strings, comments, JSON, and Markdown are never edited.
- A tsconfig that uses options TypeScript 7 removed (`baseUrl`, `moduleResolution: node10`) is rejected with the engine's diagnostic, because the engine would otherwise miss files silently. File moves are unaffected.
- Only usages in projects the engine loads are renamed. Put every caller in the package's tsconfig, then search for the old name and run the project's typecheck. Edits that would land outside `--project-path` (for example a referencing project) stop the rename.
- Use `--dry-run --json` first to see which files change and how many edits each gets.
- `rename --batch` takes N renames in one engine session: each is written after it is proven, and the earlier ones are put back if a later one fails.

The safety sequence (check, find the tools, plan, prove in memory, write through an undo journal, stop the server) is printed by `refac guide safety`.
