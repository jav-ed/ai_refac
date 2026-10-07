# Examples

These examples show the path rewrites the backend performs. Each shows the file before and after the move named above it.

## Incoming link update

Moving `target.md` to `guides/target.md`:

```md
See [Target](./target.md).
```

becomes:

```md
See [Target](./guides/target.md).
```

The `./` was there before, so it stays. A link written `[Target](target.md)` becomes `[Target](guides/target.md)`.

## Outgoing link recalculation

Moving `target.md` to `guides/target.md`:

```md
[Sibling](./sibling.md)
[Leaf](./nested/leaf.md)
```

becomes:

```md
[Sibling](../sibling.md)
[Leaf](../nested/leaf.md)
```

A path that climbs never gets `./`.

## Reference definitions, including a multi-line one

Moving `target.md` to `guides/target.md`:

```md
See [Target][target] and [Other][other].

[target]: ./target.md#deep-dive "Deep Dive"
[other]:
   target.md
```

becomes:

```md
See [Target][target] and [Other][other].

[target]: ./guides/target.md#deep-dive "Deep Dive"
[other]:
   guides/target.md
```

## Raw HTML

Moving `docs/logo.png` to `assets/logo.png`:

```md
<img src="docs/logo.png" width="64">
<img srcset="docs/logo.png 1x, docs/logo@2x.png 2x" src="docs/logo.png">
```

becomes:

```md
<img src="assets/logo.png" width="64">
<img srcset="assets/logo.png 1x, docs/logo@2x.png 2x" src="assets/logo.png">
```

## Folders

Moving `docs` to `documentation` rewrites every link into the folder:

```md
[Guide](docs/guide.md#install) [All docs](docs/) ![Logo](docs/img/logo.png?raw=1)
```

becomes:

```md
[Guide](documentation/guide.md#install) [All docs](documentation/) ![Logo](documentation/img/logo.png?raw=1)
```

Links between files inside the folder are not touched: they moved together.

## Spaces and percent-encoding

Moving `docs` to `documentation`:

```md
[a](<docs/My Notes.md>) [b](docs/My%20Notes.md)
```

becomes:

```md
[a](<documentation/My Notes.md>) [b](documentation/My%20Notes.md)
```

## A link to code

Moving `src/app.py` to `src/main.py` with the Python backend, in the same command:

```md
Start in [app](src/app.py#L10).
```

becomes:

```md
Start in [app](src/main.py#L10).
```

## Left alone

```md
[External](file:///tmp/guide.md)
[Docs](https://example.com/guide)
[Anchor](#install)
`[code](docs/guide.md)`
```

None of these change when `docs/guide.md` moves.
