# Go

The Go backend uses `gopls` (the official Go language server) for two operations. **Symbol rename** (`refac rename` on a `.go` file: variables, functions, methods, fields, types) is described in [Go symbol rename](../Symbol_Rename/go.md). **File moves** (`refac move`) use `textDocument/rename` on the package name symbol, described below. Because Go's package-per-directory model ties all files in a directory to a single package, renaming any one file's package also renames the whole package.

## Required tooling

- `gopls`, found by the shared locator: `REFAC_GOPLS`, then `PATH`, then `$GOBIN`, `$GOPATH/bin`, `~/go/bin`. When it is missing the error lists those places and names `refac doctor go`, which prints the install command (`go install golang.org/x/tools/gopls@latest`). See [Language servers](../../Setup/language_Servers.md).

## How it works

### Cross-directory move

A cross-directory move changes the package the file belongs to. The driver:

1. Reads the `go.mod` at the project root to derive the target package import path.
2. Issues a `textDocument/rename` on the `package` declaration in the source file, supplying the full target import path as the new name.
3. `gopls` returns a workspace edit with text edits (updating all importers) and `RenameFile` resource ops (physically moving the files it controls).
4. The driver applies the workspace edit, then completes any remaining filesystem moves.

Because gopls renames the **entire package** when any file is renamed, all `.go` files in the source directory are moved together. The CLI output notes any collateral files relocated beyond what was explicitly requested.

### Same-directory move (rename only)

When source and target are in the same directory, no package rename is needed. The file is moved on the filesystem; no LSP call is made.

### Batch session architecture

When a batch contains files from multiple source packages, all cross-directory renames are sent to **one gopls session**. Each package rename fires as a sequential `textDocument/rename` within that session, with `textDocument/didChange` notifications sent between renames to keep gopls's view current. This is O(1) gopls startups regardless of how many packages are in the batch.

### Dry run (`move --dry-run`)

The same package-rename requests are sent to gopls and its answer is read instead of applied (`LspClient::plan_symbol_renames`): the preview lists the requested paths, the `RenameFile` operations gopls added (the other files of the package, with a note), and the text edits per file. Several package renames in one command are each asked against the files as they are, not as the one before left them (the real move writes each before it asks the next); two renames whose edits touch the same text are refused with that reason, since the second answer would be wrong. A move inside one directory needs no server and previews as the plain move.

## Known limits

- **Whole-package moves only.** Partial-package moves (moving some but not all files from a directory to different destinations) are not supported. gopls moves all files in the package together on the first rename; subsequent files from the same source dir are handled by the filesystem step alone.
- **`go.mod` required for cross-directory moves.** The driver reads `go.mod` at the project root to construct the target import path. Without it, cross-directory moves will error.
- **gopls must be installed.** Without gopls, cross-directory moves cannot update importers in other packages. Same-directory moves (renames) still work without gopls because no import path changes.
