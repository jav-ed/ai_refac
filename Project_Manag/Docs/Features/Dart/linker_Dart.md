# Dart

Dart has two operations, both on the analysis server in the Dart SDK (`dart language-server`):

- **File moves** (`refac move`): the server's `willRenameFiles` rewrites the imports and `package:` URIs that point at the moved file. The plan is checked before anything is written: a plan that would leave an import pointing at a file that will not exist is refused with the imports listed. The server answers with a partial plan while it is still analysing (run the move again), and it rewrites `package:` imports only when `.dart_tool/package_config.json` exists (`dart pub get`). The code is [`src/drivers/dart.rs`](../../../../src/drivers/dart.rs) and `src/drivers/dart/directives.rs`.
- **Symbol rename** (`refac rename`): variables, methods, classes, fields, and named parameters with every reference. See [Dart symbol rename](../Symbol_Rename/dart.md).

## What you need

The Dart SDK (`dart`) and, in the project, `pubspec.yaml` with `dart pub get` having run. Where the SDK is looked for, and the install steps, are printed by `refac doctor dart`; see [Language servers](../../Setup/language_Servers.md).

## Tests

`src/drivers/dart.rs` holds the move tests (`test_dart_move_updates_imports` runs when `dart` is available), `tests/moves/dart_package_config.rs` the `package_config.json` refusal, and `tests/rename/dart.rs` the rename scenarios ([Dart symbol rename](../Symbol_Rename/dart.md)).

## Dry run (`move --dry-run`)

The preview is the plan the real move uses: the analysis server's answer to `workspace/willRenameFiles`, checked for imports that would dangle, and counted instead of applied. The edits per file are the server's text edits (one per rewritten import or export). A plan that would leave an import pointing at a missing file is refused as in a real move.
