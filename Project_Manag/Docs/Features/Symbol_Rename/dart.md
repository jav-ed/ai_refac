# Dart symbol rename

`refac rename` on a `.dart` file renames a class, method, getter, field, named parameter, function, constant, or local with the analysis server in the Dart SDK (`dart language-server`), across the package, including `export ... show` lists, `package:` imports, named arguments, field formals (`this.width`), and `[Name]` links in doc comments.

```bash
refac rename --project-path /path/to/package --file lib/shapes.dart \
  --symbol area --new-name surface --line 7
```

`--project-path` must hold `pubspec.yaml` **and** `.dart_tool/package_config.json`. Without the second file the server cannot resolve `package:` imports, so it would silently miss every file that imports the symbol; the command stops before any server starts and says "Run `dart pub get`". Flutter projects use `flutter pub get`. The SDK is found through `REFAC_DART`, `PATH`, or `$FLUTTER_ROOT/bin/cache/dart-sdk/bin`; if it is missing the error names `refac doctor dart`.

## What the server does by itself

- **Renames override hierarchies.** Renaming `Shape.area` renames the overriding methods in `Rect` and `Circle` and the calls through any of them, so refac needs no family step (unlike Python).
- **Follows `package:` and relative imports, `export ... show` combinators, and named arguments** at every call.
- **Renames doc-comment links** (`[Shape.area]`). Those edits sit on a `///` line; the engine accepts an edit on a comment line that replaces exactly the old name.
- **Refuses true redeclarations** itself; the capture case (a local named like a parameter) is refused by the server or by the proof.

## What refac adds

- **Waits for the analysis of every changed document before it checks the result.** The proof shows the server the renamed text and asks for the references again. The Dart server takes a changed document in without analysing it, and its status notifications arrive after the answers they concern, so asking at once returned references that were missing the files not yet analysed. Refac now asks for the semantic tokens of each changed document first, a request the server can only answer once that document is fully resolved ([Engine](engine.md): `settle`). The symptom without it: five or six of the nine scenarios failed verification on every run, always on usages in files that were not open before ("usages that no longer refer to the symbol: report.dart:4:45").
- **Points an import path to `refac move`**, as for Python.
- **Reports what is still written**: a call on a `dynamic` value (`thing.area()` in `lib/report.dart`) is not linked to the symbol, and the note lists it with its line.

## Limits

- Reflection through `runtimeType` prints the new name of a renamed class; the tests expect exactly that.
- Generated files (`*.g.dart`) are renamed only where the server sees them; regenerate afterwards.
- A package inside a workspace needs one call per package root.
- A class or function that other packages in the pub cache use cannot be renamed: the command stops and names those uses, because the server never edits dependencies (measured on `QueueList` of the `collection` package, used by `async`).

## Tests

`tests/dart_rename.rs` (nine scenarios, eight `#[ignore]`d, fixture `tests/fixtures/dart/rename_package`, package `shop`; the tests run `dart pub get --offline` in the temporary copy): a method with overrides and the doc-comment link (the `dynamic` call is left and reported, then fixed by hand), a function through `export ... show`, a field with its field formal and named arguments, a class through exports and imports, a named parameter with its label at every call, a name shared by two locals, a capture (refused), a dry run, and, without a server, a project without `package_config.json` (refused with `dart pub get`). Each successful scenario runs `dart analyze --fatal-infos` and the program. Run with the Dart SDK installed: `cargo test --test dart_rename -- --include-ignored`.
