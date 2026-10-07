# Limits And Gaps

This file owns the current boundaries of Markdown support.

## Hard Limits

- No directory moves. Only single-file `.md` moves are supported.
- No support for non-`.md` source files in the Markdown backend.
- Only Markdown files are scanned for Markdown-link updates. Non-Markdown files are not rewritten when they point to moved Markdown files.

## Unsupported Markdown Forms

- Wiki-links such as `[[Page]]`
- HTML links such as `<a href="...">`
- HTML images such as `<img src="...">`
- Autolinks such as `<https://example.com/doc>` (they are URLs, not project paths)

## Parsing Model

- A CommonMark parser (`pulldown-cmark`, GFM tables and footnotes on) decides what is a link, an image, or a reference definition. Links inside fenced and indented code blocks, inline code spans, HTML comments, raw HTML, and YAML or TOML front matter are therefore left alone, and a reference definition may continue on the next line.
- refac itself only reads the destination: the parser reports where a link or definition sits, and `src/drivers/markdown/parser/destination.rs` narrows that to the destination text, so the rewrite replaces those bytes and nothing else. Any structure it does not expect is an error.
- It is not a Markdown refactoring engine beyond destinations.
- It updates destination paths. It does not rename labels, link text, headings, or anchors.
- Reference-style usage forms keep working because the definition destination is updated, not because the usage text is transformed.

## Coverage Gaps

- The implementation has focused integration coverage for single-file Markdown moves.
- Batch Markdown moves are wired through the CLI and backend map logic, but they do not yet have dedicated end-to-end coverage for multiple inter-referencing Markdown files in one command.
