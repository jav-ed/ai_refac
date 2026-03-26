# Python

The Python backend runs two engines — Rope (primary) and Pyrefly (fallback). Both are tried in order; the first to succeed wins.

## Required tooling

The driver checks for Python in this order:
1. `.venv/bin/python` (relative to the working directory — preferred)
2. System `python3`

`rope` must be importable from whichever Python is found. Pyrefly is only needed as a fallback if Rope is absent or fails at runtime.

The backend is available if **either** engine is reachable.

## Engine selection

| Order | Engine | Why |
| :--- | :--- | :--- |
| 1st | Rope | More reliable import rewriting for file moves |
| 2nd (fallback) | Pyrefly | Used if Rope is absent or raises an error |

Rope is preferred because the Pyrefly `willRenameFiles` LSP path is less reliable for move operations in practice. If Rope fails at runtime (not just missing), a warning is logged and Pyrefly is tried automatically.

## Known limits

- Both engines require `__init__.py` files to resolve package boundaries correctly. Projects without them (namespace packages) may see incomplete import updates.
- **Rope does not trace through `__init__.py` re-exports.** If a module re-exports a symbol and callers import via the re-export (e.g. `from myapp.utils import format_date` where `utils/__init__.py` does `from .formatters import format_date`), those indirect callers are not updated. They continue to work at runtime because `utils/__init__.py` itself is updated, but the import path in the caller file is not changed.
- Pyrefly is invoked via the LSP `willRenameFiles` notification path — it does not guarantee that all importers are updated in every project layout.
