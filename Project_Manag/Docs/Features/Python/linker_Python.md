# Python

The Python backend runs two engines — Rope (primary) and Pyrefly (fallback). Both are tried in order; the first to succeed wins.

## Required tooling

- `python3`
- `rope` Python package (`pip install rope`)
- `pyrefly` Python package (`pip install pyrefly`) — only needed if Rope is unavailable or fails

The backend is available if **either** engine is installed. Having both is safest.

## Engine selection

| Order | Engine | Why |
| :--- | :--- | :--- |
| 1st | Rope | More reliable import rewriting for file moves |
| 2nd (fallback) | Pyrefly | Used if Rope is absent or raises an error |

Rope is preferred because the Pyrefly `willRenameFiles` LSP path is less reliable for move operations in practice. If Rope fails at runtime (not just missing), a warning is logged and Pyrefly is tried automatically.

## Known limits

- Both engines require `__init__.py` files to resolve package boundaries correctly. Projects without them (namespace packages) may see incomplete import updates.
- Pyrefly is invoked via the LSP `willRenameFiles` notification path — it does not guarantee that all importers are updated in every project layout.
