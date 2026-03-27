---
title: Markdown
description: How refac handles Markdown file moves using its native Rust backend.
---

The Markdown backend is implemented natively in Rust. It requires no external tooling and handles all Markdown link rewriting internally.

## What it does

When a Markdown file moves, `refac`:

- Updates **incoming links** — other `.md` files that point to the moved file have their link destinations rewritten.
- Updates **outgoing links** — relative links inside the moved file are recalculated against its new location.

Both directions are handled in one operation.

## Supported link forms

| Form | Supported |
|---|---|
| Inline links: `[text](./path.md)` | ✅ |
| Inline links with anchors: `[text](./path.md#section)` | ✅ |
| Inline images: `![alt](./image.png)` | ✅ |
| Reference definitions: `[id]: ./path.md` | ✅ |
| Reference definitions with anchors/titles: `[id]: ./path.md#s "Title"` | ✅ |
| Angle-bracket destinations: `[id]: <../path.md>` | ✅ |
| Wiki-links: `[[Page]]` | ❌ |
| HTML links: `<a href="...">` | ❌ |
| HTML images: `<img src="...">` | ❌ |

## Path handling

- Relative paths are rewritten as relative paths from the file's new directory.
- Same-directory targets are emitted with `./`.
- Anchor fragments (`#section`) are preserved when the path portion changes.
- External URIs (`https://`, `file://`, etc.) are left unchanged.

## Limitations

- No directory moves. Only individual `.md` files are supported.
- Only `.md` files are scanned for link updates. If a non-Markdown file links to a moved `.md` file, it is not updated.
- Links inside fenced code blocks and inline code spans are not rewritten.

See [Limits & Gaps](/languages/markdown/limits/) for the full boundary list, and [Examples](/languages/markdown/examples/) for before/after scenarios.
