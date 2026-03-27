---
title: Limits & Gaps
description: Boundaries and unsupported constructs in the Markdown backend.
---

## Hard limits

- **No directory moves.** Only individual `.md` files are supported as sources.
- **`.md` files only.** The backend only scans Markdown files for link updates. Non-Markdown files (HTML, config files, etc.) that link to a moved `.md` file are not updated.
- **No code block rewriting.** Links inside fenced code blocks and inline code spans are not rewritten — they are treated as literal text.

## Unsupported Markdown forms

| Form | Status |
|---|---|
| Wiki-links: `[[Page]]` | Not supported |
| HTML links: `<a href="...">` | Not supported |
| HTML images: `<img src="...">` | Not supported |
| Autolinks: `<https://example.com/doc>` | Not supported |
| Multiline reference definitions | Not supported |
| Multiline reference titles | Not supported |

## Parsing model

The implementation is path-oriented, not a full CommonMark AST. It:

- Updates **destination paths** in links and reference definitions.
- Does **not** rename labels, link text, headings, or anchor names.
- Reference-style usages (`[Text][id]`, `[id][]`) continue to work because the definition destination is updated, not because the usage text is transformed.

## Coverage gaps

Batch Markdown moves (multiple `.md` files in one `refac move` call) are wired through the CLI and backend but do not yet have dedicated end-to-end test coverage for multiple inter-referencing files moved together.
