---
title: Python
description: How refac handles Python file moves using Rope and Pyrefly.
---

The Python backend runs two engines in order — Rope (primary) and Pyrefly (fallback). The first to succeed wins. Both are tried automatically; no configuration is needed.

## Required tooling

The driver looks for Python in this order:

1. `.venv/bin/python` relative to the working directory (preferred)
2. System `python3`

`rope` must be importable from whichever Python is found. Pyrefly is only needed as a fallback if Rope is absent or fails at runtime. The backend is available if either engine is reachable.

```bash
pip install rope
# or, for the fallback only:
pip install pyrefly
```

## Engine selection

| Order | Engine | Why |
|---|---|---|
| 1st | Rope | More reliable import rewriting for file moves |
| 2nd | Pyrefly | Used if Rope is absent or raises an error at runtime |

Rope is preferred. If Rope fails at runtime (not just missing), a warning is logged and Pyrefly is tried automatically.

## Limitations

**`__init__.py` re-exports:** Rope does not trace imports that go through `__init__.py` re-exports. If a package re-exports a symbol and callers import via the re-export (e.g. `from myapp.utils import format_date` where `utils/__init__.py` does `from .formatters import format_date`), those indirect callers are **not updated**. They continue to work at runtime because `utils/__init__.py` itself is updated, but the import path in the caller file is not changed.

**Namespace packages:** Both engines require `__init__.py` files to resolve package boundaries correctly. Projects without them (namespace packages) may see incomplete import updates.

**Pyrefly path:** Pyrefly is invoked via the LSP `willRenameFiles` notification — it does not guarantee that all importers are updated in every project layout.
