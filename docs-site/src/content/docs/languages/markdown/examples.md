---
title: Examples
description: Before and after examples of Markdown link rewrites.
---

These examples show what the Markdown backend rewrites when a file moves.

## Incoming link update

Moving `target.md` to `guides/target.md`. A file that links to it is updated:

**Before:**
```md
# Index

See [Target](./target.md).
```

**After:**
```md
# Index

See [Target](./guides/target.md).
```

## Outgoing link recalculation

Moving `target.md` to `guides/target.md`. Links inside the moved file are recalculated:

**Before:**
```md
# Target

[Sibling](./sibling.md)
[Leaf](./nested/leaf.md)
```

**After:**
```md
# Target

[Sibling](../sibling.md)
[Leaf](../nested/leaf.md)
```

## Reference definition update

Moving `target.md` to `guides/target.md`. The reference definition destination is updated; the usage text is unchanged:

**Before:**
```md
# Overview

See [Target][target].

[target]: ./target.md#deep-dive "Deep Dive"
```

**After:**
```md
# Overview

See [Target][target].

[target]: ./guides/target.md#deep-dive "Deep Dive"
```

## External URI preservation

External URIs are never rewritten, regardless of the move:

**Before:**
```md
# Target

[External](file:///tmp/guide.md)
[Docs](https://example.com/guide)
```

**After:**
```md
# Target

[External](file:///tmp/guide.md)
[Docs](https://example.com/guide)
```
