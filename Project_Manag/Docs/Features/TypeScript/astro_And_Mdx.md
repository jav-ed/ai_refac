# Astro and MDX

`.astro` components and `.mdx` pages are code that imports other code, and most Astro sites have more of them than plain `.ts` files. Moves and symbol renames reach them as follows. The decision and the measurements behind it are in [Astro options](../../Investigation/astro_Options.md).

| Operation | `.ts` / `.js` | `.astro` | `.mdx` |
|---|---|---|---|
| Move the file | ✅ importers in all three kinds of file follow | ✅ its own imports (frontmatter, bundled scripts, assets) and its importers follow | ✅ its own `import`/`export` lines, its frontmatter paths, its Markdown links and its importers follow (the Markdown driver moves it) |
| Move a picture or page that code imports | n/a | ✅ imports in `.ts`, `.astro`, `.mdx` and `url()`/`@import` in `<style>` blocks and `.css` files follow | ✅ same |
| Move a `.css` file | ✅ importers (`import`, `@import`, `<style>`) and its own `url()`/`@import` follow | | |
| Move a file or folder that code names by a string | ✅ `base: "./src/Content/blog"` of a content collection, a font in a config object, `import.meta.glob` patterns | ✅ | ✅ |
| Move a picture a page names in its frontmatter, or the page | n/a | n/a | ✅ `image: ../assets/hero.webp` follows the picture, and the page keeps naming it from its new folder (`.md` pages too) |
| Rename a symbol exported from `.ts` | ✅ | ✅ with `--astro` (see below), otherwise refused | refused, the files are listed |
| Rename a symbol declared in `.astro` | n/a | ✅ with `--astro` | n/a |

## Moves

The move engine (`scripts/ts_refactor.ts`, Oxc) reads three kinds of file:

- **`.ts`, `.js` and the rest**: as before.
- **`.astro`**: the frontmatter, every `<script>` without attributes, and the `src` of a `<script>` whose only attribute is `src` (Astro bundles it as an import), parsed by Astro's own compiler (`@astrojs/compiler-rs`, a locked dependency of `scripts/`). A `<script is:inline>`, `<script type="module">`, `<script async src>` or `<script define:vars>` is not bundled by Astro and is left alone (checked with `astro build`). Offsets are exact, also after emoji and Arabic text.
- **`.mdx`**: the `import` and `export` blocks at the top level, not frontmatter, code fences or prose. A block runs to the next blank line, and further while it is incomplete (a blank line inside an object does not end it).

A file the engine cannot read (a syntax error in a frontmatter) stops the move before anything is written, and the error names the file. Skipping it would leave its imports on the old path without a word.

A folder that holds only stylesheets (and no code of another language) is moved by this engine too; with pictures or pages next to the stylesheets it is a document folder, moved by the Markdown driver with the engine fixing the stylesheets' paths and their importers first. A folder with `.astro` files is code, not a "document folder": it used to be moved by the Markdown driver and reported as a success with its importers broken.

**Stylesheet paths.** `url(...)` (quoted or not) and `@import "..."` in the `<style>` blocks of `.astro` files and in plain `.css` files are paths too, and they are rewritten like imports: relative to the file, quotes kept, and a bare `url(logo.png)` stays bare when it can (`url(img/logo.png)`). The reader skips addresses, site-root paths (`/logo.png`, the `public/` folder), `#fragments`, `data:` URIs, anything a preprocessor builds (`$var`, `#{}`, `var()`) and text that is only in a comment or a string. A bare path that does not resolve is left alone (it may be an alias or a Sass partial); a `./` or `../` path that does not resolve stops the move of its file, as an import would. A path with a CSS escape stops the move, and so does a new path that cannot be written in an unquoted `url()` (a space), with the remedy in the message. A stylesheet imported without its extension (`@import "../Base/main"`, which Vite resolves to `main.css`) follows the file and is still written without it.

`.astro` and `.mdx` imports are resolved like any other asset (aliases, `baseUrl` and `paths` of the tsconfig are honoured) and keep their spelling: an alias import stays an alias, a relative one stays relative.

### Paths written as text

Some code names a file or folder by a string, not by an import. These follow a move too:

- **A string that is the path, from the project root, of a moved file or folder**, in the code of a `.ts`, `.js`, `.astro` (frontmatter, bundled scripts) or `.mdx` file: the `base` of a content collection (`base: "./src/Content/blog"`), a font in a config object, a path a build script reads (`"src/Assets/Imgs/"`). The string counts only when it is exactly such a path *and* it exists, so words, URLs, absolute paths, `..` paths, globs and text with spaces are never touched. A leading `./` and a trailing `/` stay. The path from the root does not depend on where the file holding the string is, so moving that file changes nothing here.
- **`import.meta.glob` and `Astro.glob` patterns** that start with `./`, `../` or `/`: the folder part follows a moved folder (and, for a relative pattern, the file that holds it), the rest of the pattern (`/*.webp`) is kept. A pattern with an alias, a pattern of a call with a `base` option, and a folder that does not exist are left alone. The keys of a glob's result are the paths its pattern names, so they change with it: code that writes such a key by hand (`photos["../assets/team/a.webp"]`) is not rewritten; a note says that the keys changed. A file that moves out of the folder a glob reads, or into it, leaves or joins the glob; no edit can prevent that, so it is reported (see *Notes after a move*).
- **Frontmatter values of `.mdx` and `.md` pages** that are relative paths (`./` or `../`) to an existing file or folder: the `image()` fields of a content collection (`image: ../../assets/hero.webp`, `src:` of a nested object, a list item), plain or quoted. Astro reads them relative to the page, and the Markdown driver does not treat frontmatter as links. They follow a moved target, and a moved page keeps naming the same files from its new folder. A value that names nothing (`canonical: ../about`), a URL, a site-root path and text that only mentions a path are not touched. A path the YAML value cannot hold (`#`, a quote, a backslash) is an error that names the remedy. In an `.mdx` page a frontmatter that is never closed is an error; in an `.md` page a leading `---` that never closes is an ordinary line.
- A folder of pages or pictures that the Markdown driver moves and code names by its folder name starts the engine too (see below).

### Pages and pictures that code imports

`import art from "../assets/hero.svg"` in a component and `import Card from "../Card.astro"` in an `.mdx` page break when `hero.svg` or the page moves. The Markdown driver still moves pages, pictures and folders of them, and fixes their Markdown links exactly as without code. When code uses them, the TypeScript engine runs first, plans from the files where they still are, fixes the importers and the moved page's own imports, and renames nothing (`--external` of the helper); the Markdown driver then moves the files. A move that code does not touch never starts the engine.

"Used by code" is decided by reading the project's code files and pages: a file name written as the end of a quoted path (`"./hero.svg"`, `'../a/hero.svg?url'`) or of a frontmatter line (`image: ../a/hero.webp`), the name of a moved folder written in a path (`"./src/Content/blog"`, `"./src/Content/blog/ar"`), or a moved page with `import`/`export` lines or relative frontmatter paths of its own. A Markdown link to the file is not a use; the Markdown driver fixes it. The reading can say "used" for a file that is not (a quoted attribute); the engine then finds nothing to change. A project that has an `import.meta.glob` or `Astro.glob` call starts the engine for every move of a page or picture, because a glob can match any file without naming it; the engine then reports the files that enter or leave a glob (see below).

If the engine cannot fix the code, nothing has moved: the whole request fails. The Markdown driver's plan is made first, so a move it would refuse is refused before the code changes.

## Symbol rename

The TypeScript 7 language server reads `.ts` files. It reads `.astro` files only through a content mapper, which is code from the project's `node_modules`.

**Without `--astro`**

- A project whose `tsconfig.json` lists `contentMappers` is refused with a message that names `--astro`.
- A rename that `.astro` or `.mdx` files would not follow is refused before anything is written. The files are the ones that import from a file the rename edits and write the name somewhere in their text. They are listed (twenty, then a count), because a property or method is used in a template without being imported by that name.

**With `--astro` (or `REFAC_ASTRO=1`)**

The engine starts with `runExternalCode`, tells the server that `.astro` is handled by a mapper, and renames inside the frontmatter, the template and the bundled scripts. The rename is still planned, proven in memory and written with rollback. Requirements, each one a hard error with its remedy when missing:

- Node.js on the `PATH` (the mapper is a Node program).
- `@astrojs/ts-content-mapper` installed in the project and listed in `tsconfig.json`:
  `"contentMappers": [{ "package": "@astrojs/ts-content-mapper", "extensions": [".astro"] }]`
- A tsconfig that TypeScript 7 accepts: no `baseUrl` and no `moduleResolution: node10` (TS5102, TS5090). Move `baseUrl` into `paths` entries that start with `./`.
- The engine must list at least one `.astro` file of a project that has some; if it lists none, the rename would silently skip all of them, so it stops.

`.mdx` files are still out of reach: a rename whose `.mdx` users the server cannot read is refused with the files named. Rename the symbol in them by hand, or do all of it by hand.

### Versions

The rename engine is the **TypeScript 7.1 beta** (`typescript-native`, an exact version in `scripts/package.json`) and the content mapper is 0.2.0. Both are young. Check now and then whether newer versions are out (`npm view typescript dist-tags`, `npm view @astrojs/ts-content-mapper version`), update the exact versions, run `bun install` in `scripts/`, and run the tests below.

## Notes after a move

A note is printed after the move (and in the preview of `--dry-run`) when refac knows that something is not exactly as before but cannot fix it. None of them stops the move.

- **A glob gains or loses a file.** `src/assets/team/a.webp is selected by the glob "../assets/team/*.webp" in src/components/Team.astro and will not be after the move.` and `src/assets/team/b.webp will be selected by the glob "../assets/team/*.webp" in src/components/Gallery.astro; src/other/b.webp was not.` Membership is computed with `Bun.Glob` (close to the `picomatch` that Vite uses; an exotic pattern could be judged differently) over every pattern of the call, excludes (`!pattern`) included; a renamed file that stays inside the pattern says nothing. Twenty lines at most, then `...and N more`.
- **A glob pattern was rewritten.** The keys of the result changed with it; code that writes a key by hand needs the same change.
- **Files still name a path that moved.** After the move, refac reads the text files of the project that are not code the engine edits (`.json`, `.jsonc`, `.json5`, `.yaml`, `.yml`, `.toml`, `.txt`, `.html`, `.htm`, `.xml`, `.cfg`, `.ini`, `.conf`, `.sh`, `.bash`, `.just`, `.mk`; files the project's ignore files exclude, `.git`, `node_modules`, files with `lock` in the name and files over 1 MB are skipped; hidden folders are read) and lists those that still contain the old path of a moved file or folder: `These files still name a path that moved and were not changed: astro.config.mjs (src/Assets/Fonts), ...`. The path must end where a path ends, so `src/Content/blog-old` is not a use of `src/Content/blog`. Ten files at most, then `, and N more`. This is how a `file()` loader's JSON file or a deploy script shows up.
- **Aliased asset imports were not checked.** `N rewritten import(s) of asset files through an alias could not be checked ...` when an alias import was rewritten but the resolver could not prove that it reaches the file.

## Known limitations and backlog

Each item says what refac does, why it was left, and what it would take. Most can be solved with the pieces that exist (the path-string scanner, the frontmatter reader, the glob membership code and the note channel). Search for the old path after a move and run the site's build; a passing build does not prove the CSS and public URLs are right.

The sites this was built for are Astro with MDX content, without Vue or Svelte; the owner decided that Vue and Svelte come later. Within "Not done yet" the items are listed by estimated value for such a site, most useful first.

### Left alone on purpose

- **Template attributes** (`<img src="../a.png">`, `srcset`, `poster`, `<link href>`). Astro copies them to the page as written (checked with `astro build`): they are URLs of the page, not paths from the file, so a move does not make them wrong, and rewriting them would break the page. The one template path Astro reads as a file, a bundled `<script src>`, is rewritten. *Revisit when:* Astro starts resolving these attributes (watch the Astro changelog), or a project uses an integration that does.
- **Runtime-built paths** (`` `./img/${name}.webp` ``, `path.join(dir, file)`). The text does not contain the path; only running the code does. No static tool can follow them. The "still name a path" note covers the part that is a literal folder.

### Reported, not fixed

- **Glob membership.** A file moving into or out of a folder a glob reads changes what the glob matches; the move is still done and a note says which pattern. *To fix:* nothing can be edited to keep the file in the glob; the owner has to choose a pattern or a different target.
- **Hand-written glob keys** (`photos["../assets/team/a.webp"]`). Needs the type of the glob result to find the places that index it; the note is the cheap form. *To fix:* a TypeScript-LSP "find references" on the glob's variable, then a rewrite of string-literal index expressions.
- **Config and data files that name a moved path** (`.json`, `.yaml`, `.toml`, `.html`, scripts). Listed by the "still name a path" note, not edited, because the format decides what a path is. *To fix, in this order:* a JSON/YAML scalar scanner (the frontmatter scanner in `scripts/TypeScript/Embedded/frontmatter.ts` already reads YAML scalars) for the files an Astro `file()` loader or `astro.config` reads; then, if a project needs more file types, an opt-in list of extensions for the scan.

### Not done yet, solvable

- **`public/` URLs** (`/logo.png`, `url(/fonts/a.woff2)`, `<img src="/logo.png">`). They are site URLs, not file paths, so a move inside `public/` does not touch any import. *To fix:* when a moved file is inside `public/`, build the old and new URL (`/<path below public>`) and rewrite exact quoted occurrences in code, templates, CSS and Markdown, with the same "existing path" proof as the path strings. The base path of `astro.config` (`base: "/docs"`) has to be added in front. A mention in prose or in a data file stays a note.
- **Content-collection ids.** The id of an entry follows the file name (`blog/hello-world`), and pages link to it by id in code (`getEntry("blog", "hello-world")`, `<a href={`/blog/${entry.id}`}>`). A rename of a content file changes the id and the URL. *To fix:* read the collection's `loader` (`glob({ base, pattern })`), compute old and new ids with the loader's rule, and rewrite `getEntry`/`reference()` literals with the old id plus add a note for the URL.
- **`tsconfig` `paths`, `include` and `exclude`, and other configs** that name a folder (`"@assets/*": ["src/assets/*"]`, `include: ["src/content/**"]`). A moved folder leaves them stale, and the engine then no longer sees files as callers. *To fix:* a JSONC edit that keeps comments and formatting (a parser with exact offsets, as the engine already uses for code), limited to string values that are exactly an existing path or a path with a glob tail, the same rule as the glob folders.
- **Frontmatter forms the scanner does not read.** Flow style (`image: { src: ../a.webp }`, `[../a.webp]`), multi-line scalars (`|`, `>`), and anchors and aliases (`&x`, `*x`). Only plain and quoted block scalars are read. *To fix:* extend the scanner in `Embedded/frontmatter.ts`; keep the rule "a value that names no existing file is not touched" so a wrong parse changes nothing.
- **`.mdx` rename.** The language server cannot read `.mdx`, so a rename with `.mdx` users is refused and the files are listed. *To fix:* an `.mdx` content mapper for the language server (check whether one has appeared), or a rename of the identifier in the `import`/`export` blocks and the expressions of the page with the Oxc parser (the engine already finds those blocks for moves); the JSX in the prose would need its own reading.
- **Sass and Less** (`.scss`, `.sass`, `.less`). They are not moved by refac ("Skipped (unsupported extension)"), and `url()`/`@import`/`@use` paths in them are not read when a picture moves. Partials (`_base.scss` imported as `base`) and load paths make a path not a file path. *To fix:* add the extensions to the stylesheet reader with the partial rule (`dir/_name.scss`, `dir/name/_index.scss`) and treat `@use`/`@forward` like `@import`; the check by resolution in `verifyPlan` needs a resolver mode for them.
- **Vue and Svelte** (`.vue`, `.svelte`). They keep pointing at a moved module. Owner decision: later. *To fix:* the same shape as `.astro`: a script-block reader per format (`<script>` / `<script setup>` / `<script lang="ts">`, style blocks) with offsets, the router rule for `.vue`/`.svelte` in `CODE_EXTENSIONS`, and and, for rename, a content mapper for the language server like the Astro one (candidates to evaluate: Volar's TypeScript plugin for Vue, `svelte2tsx` for Svelte).

### Verification limits

- The 50-item sweep (`Trials/Astro/real_sweep.sh`) moves one item at a time on a copy of a real site, builds, moves back and compares the tree. "Build ok" means the Vite bundle step of `astro build` finished; the prerender step of that copy fails on its own (it needs an `.astro/` folder the copy does not have), so pages and links were not compared. A move that breaks a page without breaking the bundle (a URL, a content id) passes.
- A symbol rename with `--astro` was tested on fixtures only, not on the real site.
- The engine moves at most 30 files per call.
- `Bun.Glob` is called without the `dot` option, so dotfiles are not matched (picomatch's default is the same); not tested against a real folder of dotfiles.
- The Astro language server (`@astrojs/language-server` 2.17.3) crashes on folder, image and CSS moves (`TypeError` in `volar-service-typescript`). Moves do not use it, so this only matters if it is ever tried as an alternative; not reported upstream yet.

## Tests

- Engine: `scripts/Tests/TypeScript/components.test.ts` (readers, UTF-16 offsets, moves, external moves, asset formats, stylesheets, component users), `script_src.test.ts`, `path_strings.test.ts`, `globs.test.ts` and `frontmatter.test.ts`.
- Routing: `src/logic/grouping/referenced/tests.rs` and `src/logic/grouping/route/tests.rs`.
- Through the CLI: `tests/typescript/astro/` (components, pages and pictures, paths written as text, rename and `--astro`). The `--astro` tests link `@astrojs/ts-content-mapper` from `scripts/node_modules`, where it is a dev dependency for this reason.
