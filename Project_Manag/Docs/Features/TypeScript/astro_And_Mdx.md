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
- **`import.meta.glob` and `Astro.glob` patterns** that start with `./`, `../` or `/`: the folder part follows a moved folder (and, for a relative pattern, the file that holds it), the rest of the pattern (`/*.webp`) is kept. A pattern with an alias, a pattern of a call with a `base` option, and a folder that does not exist are left alone. The keys of a glob's result are the paths its pattern names, so they change with it: code that writes such a key by hand (`photos["../assets/team/a.webp"]`) is not rewritten and needs a look. A file that moves out of the folder a glob reads, or into it, leaves or joins the glob; refac does not warn about that.
- **Frontmatter values of `.mdx` and `.md` pages** that are relative paths (`./` or `../`) to an existing file or folder: the `image()` fields of a content collection (`image: ../../assets/hero.webp`, `src:` of a nested object, a list item), plain or quoted. Astro reads them relative to the page, and the Markdown driver does not treat frontmatter as links. They follow a moved target, and a moved page keeps naming the same files from its new folder. A value that names nothing (`canonical: ../about`), a URL, a site-root path and text that only mentions a path are not touched. A path the YAML value cannot hold (`#`, a quote, a backslash) is an error that names the remedy. In an `.mdx` page a frontmatter that is never closed is an error; in an `.md` page a leading `---` that never closes is an ordinary line.
- A folder of pages or pictures that the Markdown driver moves and code names by its folder name starts the engine too (see below).

### Pages and pictures that code imports

`import art from "../assets/hero.svg"` in a component and `import Card from "../Card.astro"` in an `.mdx` page break when `hero.svg` or the page moves. The Markdown driver still moves pages, pictures and folders of them, and fixes their Markdown links exactly as without code. When code uses them, the TypeScript engine runs first, plans from the files where they still are, fixes the importers and the moved page's own imports, and renames nothing (`--external` of the helper); the Markdown driver then moves the files. A move that code does not touch never starts the engine.

"Used by code" is decided by reading the project's code files and pages: a file name written as the end of a quoted path (`"./hero.svg"`, `'../a/hero.svg?url'`) or of a frontmatter line (`image: ../a/hero.webp`), the name of a moved folder written in a path (`"./src/Content/blog"`, `"./src/Content/blog/ar"`), or a moved page with `import`/`export` lines or relative frontmatter paths of its own. A Markdown link to the file is not a use; the Markdown driver fixes it. The reading can say "used" for a file that is not (a quoted attribute); the engine then finds nothing to change.

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

## Not done

These are left alone or refused. Search for the old path after a move and run the site's build; a passing build does not prove the CSS and public URLs are right.

- Template attributes are not rewritten on purpose. Astro copies `<img src>`, `srcset`, `poster` and `<link href>` to the page as written (checked with `astro build`): they are URLs of the page, not paths from the file, so moving a file does not make them wrong. The one template path Astro reads as a file, a bundled `<script src>`, is rewritten.
- `.scss`, `.sass`, `.less` and other formats that code reads are not moved by refac (the move is refused with "Skipped (unsupported extension)"), and their `url()` and `@import` paths are not read when a picture moves. Plain `.css` is.
- Content-collection ids, which change with the file name, and the membership of a glob (see above).
- `public/` URLs (`/logo.png`), which are not file paths.
- Data files that name pictures (a JSON or YAML file read by a `file()` loader) are not read.
- `.vue` and `.svelte` files are not read yet; they keep pointing at a moved module.

## Tests

- Engine: `scripts/Tests/TypeScript/components.test.ts` (readers, UTF-16 offsets, moves, external moves, asset formats, stylesheets, component users), `script_src.test.ts`, `path_strings.test.ts`, `globs.test.ts` and `frontmatter.test.ts`.
- Routing: `src/logic/grouping/referenced/tests.rs` and `src/logic/grouping/route/tests.rs`.
- Through the CLI: `tests/typescript/astro/` (components, pages and pictures, paths written as text, rename and `--astro`). The `--astro` tests link `@astrojs/ts-content-mapper` from `scripts/node_modules`, where it is a dev dependency for this reason.
