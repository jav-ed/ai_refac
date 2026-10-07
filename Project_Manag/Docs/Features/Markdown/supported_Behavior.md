# Supported Behavior

This file owns the description of what the Markdown backend does.

## Scope

- The backend is native Rust. It does not depend on an external Markdown toolchain or runtime.
- It moves Markdown files (`.md`, `.markdown`, `.mdx`, any letter case), asset files that Markdown points at, and folders of both. The assets are images (`png`, `jpg`, `jpeg`, `gif`, `svg`, `webp`, `avif`, `ico`, `bmp`, `tif`, `tiff`, `heic`), `pdf`, audio and video (`mp4`, `webm`, `mov`, `mp3`, `wav`, `ogg`, `flac`), fonts (`woff`, `woff2`, `ttf`, `otf`), and `zip`. Formats that code reads or imports (`json`, `yaml`, `toml`, `css`, `html`, `txt`, `csv`) are not assets: moving one would leave references in code behind without a word, so they are reported as skipped.
- A folder is a document folder when it holds Markdown or assets and no code. A folder with TypeScript or Kotlin is moved by those backends; a folder with any other code is refused.
- It also runs after the other backends. When a TypeScript, Python, Rust, Go, Dart, or Kotlin file or folder has been moved, the Markdown links that point at it are fixed in the same command (see "Links to code" below).
- `--project-path` may be relative (`.`); it is resolved against the working directory. It is the folder whose Markdown files are checked. Without it, the files below the folder that all moved paths share are checked.
- Files and folders are found with the ignore rules of the project: `.gitignore` and `.ignore` files inside the project path are read even where there is no git repository, hidden folders such as `.github/` and `.agents/` are searched, and `.git/` and `node_modules/` are never searched.

## Incoming updates

When a path moves, every link to it is updated, wherever the link is written. The destination forms that are read:

- Inline links and images: `[Guide](./guide.md)`, `![Logo](img/logo.png)`, with a `#fragment`, a `?query`, or both
- Reference definitions, including collapsed, shortcut, titled, angle-bracket, and multi-line ones (`[guide]:` with the destination on the next line)
- Angle-bracket destinations with spaces: `[a](<docs/My Notes.md>)`
- Percent-encoded destinations: `docs/My%20Notes.md` is the file `docs/My Notes.md`
- Raw HTML in Markdown: `href`, `src`, and `poster` values, and every URL in a `srcset`, for example `<img src="docs/logo.png" width="64">`
- Links in table cells, list items, block quotes, link text that continues on the next line, and an image inside a link
- Links to a folder (`docs/`), which follow the folder when it moves, and links to a file inside a moved folder

## Outgoing recalculation

When the Markdown file moves, or sits inside a folder that moves, its own relative destinations are measured from the new place. A link that already says the right thing is not touched: two files that move together keep their links byte for byte, even when a link was spelled oddly (`x/../a.md`).

## How a link is written

The rewrite changes the destination text and nothing else.

- The author's habits are kept. A `./` prefix is only written where the old path had one and the new path does not climb (`../x` never gets one). A trailing `/` on a folder link stays. A destination that was percent-encoded stays encoded; a space in an unwrapped destination is written as `%20`; a destination in `<...>` keeps its space.
- Characters that cannot stand in a destination are escaped (`%`, `#`, `?`, brackets, quotes); balanced parentheses (`a_(b).md`) stay as they are.
- Anchor fragments and queries are preserved when the path changes.
- Web addresses (`https://`, `mailto:` and any scheme), `#anchors`, site-root paths (`/docs/a.md`), paths with template syntax (`{{ x }}`, `${x}`), and paths with an HTML entity are left unchanged.
- Line endings, a byte-order mark, and all text around a destination stay byte for byte.

## Links to code

After a TypeScript, Python, Rust, Go, Dart, or Kotlin batch, one pass over the Markdown files of the project fixes the links to what moved, files and folders alike. A link is only changed when its old target is gone and the new one exists, so a half-moved folder never sends a link to a file that was not moved. The response gets a section `// Markdown links to the moved files:` with the number of files checked and links changed. If this pass fails, the command fails with an error that says the files were moved but the links were not updated.

## Safety

- The whole change is planned before the disk is touched: moves, then rewritten Markdown. Any failure undoes everything in reverse order (written files restored, moved files moved back), and the error says whether the rollback was complete.
- Refused before anything changes: a missing source; an existing target; a Markdown file moved to a name that is not Markdown (and the reverse); a folder moved into itself; two moves that overlap; a folder that holds code.
- Markdown files that are not valid UTF-8 are not read and not changed; the response names them so they can be checked by hand.

## Reference-style Markdown

Usage text such as `[Text][id]`, `[id][]`, or `[id]` is never renamed. The matching `[id]: ...` definition is updated, so those usages keep resolving.

## Output

The response lists each moved path, then a note such as `Checked 9 Markdown files; updated 13 links in 9 files.`
