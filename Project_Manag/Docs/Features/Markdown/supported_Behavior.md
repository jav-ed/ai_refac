# Supported Behavior

This file owns the description of what the current Markdown implementation does.

## Scope

- The backend is native Rust. It does not depend on an external Markdown toolchain.
- It only handles `.md` files.
- It only handles file moves. It does not support directory moves.
- It scans Markdown files recursively under the relevant workspace root and rewrites Markdown path destinations where needed.

## Incoming Updates

When a Markdown file moves, other Markdown files that point to that file are updated.

Supported destination forms for this:

- Inline links such as `[Guide](./guide.md)`
- Inline links with anchors such as `[Guide](./guide.md#details)`
- Reference definitions such as `[guide]: ./guide.md`
- Reference definitions with anchors or titles such as `[guide]: ./guide.md#details "Guide"`
- Angle-bracket destinations such as `[guide]: <../guide.md>`

## Outgoing Recalculation

When the Markdown file itself moves, its own relative destinations are recalculated against the new location.

Examples of destinations that are recalculated inside the moved file:

- Inline Markdown links
- Inline Markdown images
- Reference-definition destinations
- Relative paths with `#fragment` anchors

## Path Handling

- Relative paths are rewritten as relative paths from the file's new directory.
- Same-directory targets are emitted with `./...`.
- Anchor fragments are preserved when the path portion changes.
- External URI schemes such as `https://...` and `file://...` are left unchanged.

## Reference-Style Markdown

Reference-style Markdown works through destination updates on the definition line.

What that means:

- Usage text such as `[Text][id]`, `[id][]`, or `[id]` is not renamed.
- The implementation updates the matching `[id]: ...` destination so those usages still resolve.
