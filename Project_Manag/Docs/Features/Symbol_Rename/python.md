# Python symbol rename

`refac rename` on a `.py` file renames a function, class, method, property, attribute, constant, parameter, or local with basedpyright, across the files of the project, including `__init__.py` re-exports and `__all__` lists, keyword arguments, and the tests.

```bash
refac rename --project-path /path/to/project --file shop/shapes.py \
  --symbol area --new-name surface --line 12
```

`--project-path` is the folder pyright treats as the root: it reads `pyrightconfig.json` or `[tool.pyright]` there and resolves imports from it. Any folder works, so refac does not guess a root from markers. The server is `basedpyright-langserver` from `PATH`, `~/.local/bin`, or the project's `.venv/bin` and `venv/bin`, or the file named by `REFAC_PYTHON_SERVER`; if it is missing the error names `refac doctor python`, which prints `pip install basedpyright` (also `pipx` or `uv tool`). basedpyright brings its own Node.js.

## Why basedpyright, and only it

Python is typed only where the code says so, so every engine renames what it can prove. Four servers were run on the same nine renames and five clashes ([Symbol rename options](../../Investigation/symbol_Rename_Options.md)): Pyrefly and ty missed references in other files (re-exports, attributes used by other modules, properties), pyright and basedpyright found them all except the overrides of a method. basedpyright is the only one of the two that can list the overrides (`textDocument/implementation`), so it is the only one refac uses; plain pyright is reported as not usable for renames. None of the four notices a name clash: all five clashes were accepted. The proof in the engine is what refuses them.

## What refac does that the server does not

- **Renames the override family.** pyright renames the method it is asked about and its calls through that class, not the methods that override it. After the server's rename, refac asks `textDocument/implementation` at the symbol, keeps the answers that are spelled like the symbol (a class's subclasses are listed too, and are not renamed), renames each of them with the same new name, and merges the edits ([Engine](engine.md)). Each override becomes a related group that the proof checks. On the nine-case matrix the method rename (a subclass override, a `super()` call, a call through the subclass) edits exactly the required places.
- **Starting from an override renames only that override and its subclasses.** The server cannot find the base method from there, so the base and the siblings stay and are listed in the note as lines that still spell the old name. Rename from the base method to change the whole family.
- **Refuses what the interpreter calls by spelling.** `__init__`, `__enter__`, and every `__name__` are refused as the old or the new name, because nothing in the code mentions the calls and the rename would silently change the program.
- **Points a module name to `refac move`.** In `from shop.report import describe`, `report` is a file. The server says it is not a symbol and refac adds: a module or package name in an import path is renamed with `refac move`, which updates the imports.
- **Refuses clashes.** A new name that equals an existing local, parameter, member, or global changes what the code means; the usages lost or gained are listed and nothing is written.
- **Reports what is still written.** A call on a parameter without an annotation (`thing.area()`), a name in a string, a docstring, or a comment is not linked to the symbol. The note lists the lines (`shop/report.py:15`) and ends with `rg -w`. Annotating the parameter and renaming again links it.

## Limits

- Dynamic access (`getattr`, `setattr`, `__dict__`, serialization keys), and reflection such as `type(x).__name__` printing a renamed class, are text, not symbols. A rename of a class changes the printed name, and the tests expect exactly that.
- Code in `site-packages` and virtual environments is not edited, and is not scanned for leftovers.
- A project with several independent roots needs one call per root.
- This is separate from Python file moves, which use Rope or Pyrefly ([Python feature docs](../Python/linker_Python.md)).

## Tests

`tests/python_rename.rs` (ten scenarios, `#[ignore]`d, fixture `tests/fixtures/python/rename_project`, package `shop`): a base method with its overrides and typed callers (the untyped call is left and reported, then fixed by hand), a function through imports and `__all__`, a dataclass field with its keyword arguments, a class through imports and annotations, a parameter with its keyword at every call, a name shared by two locals, a capture (refused), an override renamed alone (the note lists the base), a module name (refused, points to `refac move`), and a dry run. Each successful scenario runs `app.py` and `checks/test_shapes.py` and compares the output with the original. Run with basedpyright installed: `cargo test --test python_rename -- --ignored`.
