---
title: Dart
description: How refac handles Dart file moves using the Dart analysis server.
---

The Dart backend uses the Dart analysis server via LSP. It updates both relative imports and `package:` URI imports when a file moves.

## Required tooling

The Dart SDK (`dart` binary) must be installed and available in PATH.

See [dart.dev/get-dart](https://dart.dev/get-dart) for installation instructions.

## How it works

The driver starts the Dart analysis server, issues a `workspace/willRenameFiles` request, applies the workspace edit returned by the server, then moves the file on the filesystem.

## `package:` URI imports

`package:` URI imports (e.g. `import 'package:myapp/src/formatter.dart'`) are only rewritten if `.dart_tool/package_config.json` exists at the project root.

If this file is missing, run:

```bash
dart pub get
```

Without `package_config.json`, only relative imports are updated. `package:` URI imports are left unchanged.

## Limitations

**No directory moves.** Only individual `.dart` files are supported as move sources.

**Concurrent analysis servers.** The Dart analysis server is sensitive to concurrent starts. If you are running `refac` tests, Dart tests are serialized internally to avoid this issue.
