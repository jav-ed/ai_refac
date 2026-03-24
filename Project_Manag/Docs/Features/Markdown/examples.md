# Examples

These examples show the kinds of Markdown path rewrites the current backend performs.

## Incoming Link Update

Before moving `target.md` to `guides/target.md`:

```md
# Index

See [Target](./target.md).
```

After:

```md
# Index

See [Target](./guides/target.md).
```

## Outgoing Link Recalculation

Before moving `target.md` to `guides/target.md`:

```md
# Target

[Sibling](./sibling.md)
[Leaf](./nested/leaf.md)
```

After:

```md
# Target

[Sibling](../sibling.md)
[Leaf](../nested/leaf.md)
```

## Reference Definition Update

Before moving `target.md` to `guides/target.md`:

```md
# Overview

See [Target][target].

[target]: ./target.md#deep-dive "Deep Dive"
```

After:

```md
# Overview

See [Target][target].

[target]: ./guides/target.md#deep-dive "Deep Dive"
```

## External URI Preservation

Before moving `target.md` to `guides/target.md`:

```md
# Target

[External](file:///tmp/guide.md)
[Docs](https://example.com/guide)
```

After:

```md
# Target

[External](file:///tmp/guide.md)
[Docs](https://example.com/guide)
```
