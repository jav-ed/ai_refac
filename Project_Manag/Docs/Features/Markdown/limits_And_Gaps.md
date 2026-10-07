# Limits And Gaps

This file owns the current boundaries of Markdown support.

## Not links to refac

- Wiki-links such as `[[Page]]` (a separate link model resolved by note name, as Obsidian does).
- Imports and expressions in MDX (`import Logo from './logo.png'`, `<Image src={logo} />`) and JSX props other than `href`, `src`, `poster`, and `srcset`. Their text is code.
- Links in YAML or TOML front matter (`image: ./cover.png`), in code blocks and spans, in HTML comments, and in `<pre>`, `<code>`, `<script>`, `<style>`, and `<textarea>` content. Front matter is skipped on purpose; if a site generator reads paths from it, update those by hand.
- Paths inside other file types: the `href` of an SVG, CSS `url(...)`, HTML files, JSON or YAML configuration that names a Markdown file. Only Markdown files are rewritten.
- Site-root paths (`/docs/a.md`), routes without an extension (`./guide` in a docs-site generator), and paths that contain template syntax, a backslash, or an HTML entity. They are left as they are rather than guessed.

## Boundaries of a move

- A folder with code of a language refac does not move by folder (anything other than TypeScript, JavaScript, and Kotlin, which have their own backends) is refused. Move its files one by one with their own backend.
- Files ignored by `.gitignore`/`.ignore` files in the project, `.git/`, and `node_modules/` are not searched, so a link in an ignored file is not updated. A file or folder that is itself moved is always read, ignored or not.
- Go moves whole packages; the other `.go` files that gopls moves along are not known to the Markdown pass, so Markdown links to them are not updated. The response already lists them.
- Links are measured against the project path. A target outside it is fine, but Markdown files outside it are not searched.
- There is no cross-language transaction. If the TypeScript batch fails after the Markdown batch has run, the Markdown batch stays done (each batch is atomic on its own).
- `move` has no dry-run mode (only `rename` has one). The planning code could show the changes first; it is not exposed.

## Style

- A link that had to climb out of a folder loses a `./` prefix, and moving it back cannot know the prefix was there. Moving a file out and back therefore restores every byte unless a `./` link was involved; the links lead to the same files either way.
- The tool never normalises a spelling it does not have to change: `docs//a.md` and `x/../a.md` stay until their file or target moves.

## Parsing model

- A CommonMark parser (`pulldown-cmark`, GFM tables and footnotes on, YAML and TOML front matter on) decides what is a link, an image, or a reference definition, and where it sits. Raw HTML is read by a small scanner (`src/drivers/markdown/parser/html.rs`) that keeps its place across lines and knows comments and raw-text elements.
- refac itself only reads the destination: `src/drivers/markdown/parser/destination.rs` narrows the parser's range to the destination text and checks it against the parser's own destination, so the rewrite replaces those bytes and nothing else. Any structure it does not expect is an error.
- It updates destinations. It does not rename link text, headings, or anchors. Renaming a heading and the `#slug` links to it is not implemented; see [Markdown options](../../Investigation/markdown_Options.md) for the plan.

## Coverage

- Unit tests per module (`moves`, `href`, `workspace`, `rewrite`, `apply`, the HTML scanner, the parser).
- `tests/markdown_site.rs` and `tests/markdown_site_folders.rs`: a documentation site fixture moved as a file, a rename, a folder (also deeper), an image, and a batch, with exact text, an independent link-graph check (`tests/common/links.rs`), and round trips.
- `tests/markdown_corpus.rs`: the 13 link shapes used to compare other tools, pinned exactly.
- `tests/markdown_after_code.rs`: links to TypeScript, Python, Go, and Dart files, a TypeScript folder, a mixed request, and a failed Rust move.
- `tests/markdown_safety.rs`: ignore rules, non-UTF-8 files, line endings, refusals, rollback.
- `tests/markdown_scale.rs`: 3,000 files, one folder moved in well under a second.
- `tests/markdown_move.rs`, `tests/markdown_commonmark.rs`, `tests/markdown_external_links.rs`, `tests/batch_move.rs`: the original single-file and batch scenarios.
