# Dart

Read before the first Dart move or rename, and when one fails. `package:` imports are only rewritten when the package config exists, so run `dart pub get` before the first call.

## Setup

- `--project-path`: the package folder with `pubspec.yaml`, after `dart pub get`.
- Tool: the Dart SDK, which contains the analysis server (`dart language-server`): [dart.dev/get-dart](https://dart.dev/get-dart).
- `refac doctor dart` finds it (`REFAC_DART` overrides the lookup) and proves it starts.

## Move: package URI rewriting requires package config

`package:` URI imports are only rewritten if `.dart_tool/package_config.json` exists at the project root. Without it, a move that would leave a `package:` import pointing at a missing file is refused before anything is written, and the error lists the imports.

Run `dart pub get` in the project root to generate it before calling `refac`.

## Rename a symbol

`refac rename` renames overrides, `export ... show`, named arguments, field formals, and `[Name]` doc links. It needs `.dart_tool/package_config.json` too: run `dart pub get` first or the command stops and says so. Calls on an untyped receiver are listed in the notes, not renamed. Several renames of one project go into one `rename --batch` call.
