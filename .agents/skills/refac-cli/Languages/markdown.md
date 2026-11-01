# Markdown

Read before moving Markdown files, pictures or documents, and when a link was not rewritten. Markdown is built in (CommonMark): it needs no server and nothing to install, and there is no symbol rename.

## What `move` does

`move` takes Markdown files (`.md`, `.markdown`, `.mdx`), images and other assets (png, jpg, svg, pdf, fonts, media, zip), and folders that hold only those. Every link that follows is rewritten: inline links and images, reference definitions (also multi-line), angle-bracket and `%20` destinations, `?query` and `#fragment`, folder links, and the `href`/`src`/`srcset` values of raw HTML. A link is written the way the author wrote it (no forced `./`).

The same pass runs after any other backend: when `move` has moved a TypeScript, Python, Rust, Go, Dart, or Kotlin file or folder, the Markdown links to it are fixed too, and the response shows a `// Markdown links to the moved files:` note with the counts.

## What is not rewritten

Web addresses, `#anchors`, `/site-root` paths, wiki-links `[[Page]]`, MDX imports, front matter, code blocks and spans, HTML comments, and paths inside other file types. `.gitignore`d files, `.git/`, and `node_modules/` are not searched. Markdown files that are not valid UTF-8 are named in the response and left unchanged. A folder with code of another language is refused.
