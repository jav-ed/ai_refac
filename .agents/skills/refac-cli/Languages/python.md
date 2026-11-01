# Python

Read before the first Python move or rename, and when one fails. Moves and renames use different tools: Rope moves files, basedpyright renames symbols. A Python call takes 1 to 8 s.

## Setup

- `--project-path`: the folder pyright treats as the root (`pyrightconfig.json` or `[tool.pyright]`). The rename works with any `--project-path`.
- Moves: `rope`, importable from `.venv` or `python3` (`pip install rope`). `pyrefly` is only the fallback when Rope is absent (`pip install pyrefly`).
- Rename: `basedpyright`; plain pyright is not enough (`pip install basedpyright`, or `pipx`, `uv tool`).
- `refac doctor python` finds the rename server (`REFAC_PYTHON_SERVER` overrides the lookup) and proves it starts.

## Move: re-export limits

Rope cannot trace imports that go through `__init__.py` re-exports. If a package re-exports a symbol and callers import via that re-export, those callers are not updated.

Namespace packages (directories with no `__init__.py`) may also see incomplete updates.

## Rename a symbol

`refac rename` renames overrides together with the base method (rename from the base, not from an override), `__init__.py` re-exports, `__all__` strings, and keyword arguments. Special methods (`__init__`) are refused. Calls on a parameter without an annotation are listed in the note, not renamed. A module name in an import is a path: use `refac move`. Several renames of one project go into one `rename --batch` call.
