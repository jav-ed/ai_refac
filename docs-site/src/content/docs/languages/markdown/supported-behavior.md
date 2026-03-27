---
title: Supported Behavior
description: What the Markdown backend updates during a file move.
---

## Incoming updates

When a Markdown file moves, other Markdown files that link to it are updated. The following destination forms are supported:

- Inline links: `[Guide](./guide.md)`
- Inline links with anchors: `[Guide](./guide.md#details)`
- Reference definitions: `[guide]: ./guide.md`
- Reference definitions with anchors or titles: `[guide]: ./guide.md#details "Guide"`
- Angle-bracket destinations: `[guide]: <../guide.md>`

## Outgoing recalculation

When the Markdown file itself moves, its own relative destinations are recalculated against the new location. This covers:

- Inline Markdown links
- Inline Markdown images
- Reference-definition destinations
- Relative paths with `#fragment` anchors

## Path handling

- Relative paths are rewritten as relative paths from the file's new directory.
- Same-directory targets are emitted with `./`.
- Anchor fragments are preserved when the path portion changes.
- External URI schemes (`https://`, `file://`, etc.) are left unchanged.

## Reference-style Markdown

Reference-style Markdown works through destination updates on the definition line.

- Usage text (`[Text][id]`, `[id][]`, `[id]`) is **not** renamed.
- The matching `[id]: ...` definition destination is updated so those usages continue to resolve.
