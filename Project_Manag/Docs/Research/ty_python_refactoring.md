# ty (Astral) for Python File Refactoring - Research Finding

**Date**: 2026-01-02  
**Status**: ❌ Not viable (currently)  
**Blocker**: ty lacks `workspace/willRenameFiles` LSP support

---

## Intent

Replace Pyrefly with [ty](https://github.com/astral-sh/ty) (Astral's Python type checker) as the primary Python refactoring backend. ty is 10-100x faster than mypy/Pyright and from the same team as uv and Ruff.

## Finding

**ty does NOT support the `workspace/willRenameFiles` LSP method**, which is required for updating imports when files are moved/renamed.

- ty's LSP feature table lists this as "❌ Not supported"
- Tracked in: <https://github.com/astral-sh/ty/issues/1560>
- ty only supports `textDocument/rename` (symbol renaming), not file-level refactoring

## Current Fallback

Our `PythonDriver` continues to use:

1. **PyreflyDriver** (primary) - LSP-based via `workspace/willRenameFiles`
2. **RopeDriver** (fallback) - Python `rope` library via script

## Future Action

Monitor [ty issue #1560](https://github.com/astral-sh/ty/issues/1560). Once ty implements `workspace/willRenameFiles`, we can revisit this migration.

## Reference

- ty repo cloned to: `Repos/ty/`
- ty LSP docs: `Repos/ty/docs/features/language-server.md`
