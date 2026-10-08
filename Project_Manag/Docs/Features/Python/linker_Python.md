# Python

The Python backend has two jobs with different tools. **File moves** (`refac move`) run two engines, Rope (primary) and Pyrefly (fallback), tried in order; the first to succeed wins. **Symbol rename** (`refac rename` on a `.py` file) uses basedpyright and has its own page: [Python symbol rename](../Symbol_Rename/python.md); `refac doctor python` explains how to install that server. The sections below are about moves.

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

## Dry run (`move --dry-run`)

Rope applies one move after the other and reads the files it has moved, so it cannot plan on the unchanged project. The preview runs the real Rope move on a throw-away copy of the project (the files `.gitignore` does not exclude, `.ropeproject` if there is one; 500 MiB at most, `REFAC_DRY_RUN_COPY_MAX_MB` changes the limit) and reports the difference to the original: the moves, and per file the number of changed passages. Rope's own `.ropeproject` state in the copy is not part of the answer, and the project is never touched. Paths outside the project cannot be copied, so Rope refuses them in a dry run; the dispatcher then falls back to Pyrefly, as a real move does when Rope fails. Pyrefly is planned from its `willRenameFiles` answer without a copy.

## Known limits

- Both engines require `__init__.py` files to resolve package boundaries correctly. Projects without them (namespace packages) may see incomplete import updates.
- **Rope does not trace through `__init__.py` re-exports.** If a module re-exports a symbol and callers import via the re-export (e.g. `from myapp.utils import format_date` where `utils/__init__.py` does `from .formatters import format_date`), those indirect callers are not updated. They continue to work at runtime because `utils/__init__.py` itself is updated, but the import path in the caller file is not changed.
- Pyrefly is invoked via the LSP `willRenameFiles` notification path — it does not guarantee that all importers are updated in every project layout.
