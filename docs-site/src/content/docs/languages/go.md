---
title: Go
description: How refac handles Go file moves using gopls.
---

The Go backend uses [gopls](https://pkg.go.dev/golang.org/x/tools/gopls) (the official Go language server) via LSP. Because Go's package-per-directory model ties all files in a directory to a single package, any cross-directory move renames the entire package.

## Required tooling

`gopls` — checked at `gopls` in PATH, then `~/go/bin/gopls`.

```bash
go install golang.org/x/tools/gopls@latest
```

## Cross-directory move

When you move a `.go` file to a different directory, `refac`:

1. Reads `go.mod` at the project root to derive the target package import path.
2. Issues a `textDocument/rename` on the `package` declaration, supplying the full target import path.
3. gopls returns a workspace edit covering all importers and `RenameFile` resource ops for the files it controls.
4. The driver applies the workspace edit and completes any remaining filesystem moves.

**Because gopls renames the entire package**, all `.go` files in the source directory are moved together. The CLI output notes any files relocated beyond what was explicitly requested.

## Same-directory move (rename only)

When source and target share the same directory, no package rename is needed. The file is moved on the filesystem only — no LSP call is made.

## Batch session architecture

When a batch contains files from multiple source packages, all cross-directory renames are sent to **one gopls session**. Each package rename fires sequentially within that session, with `textDocument/didChange` notifications between renames to keep gopls's view current. This means one gopls startup regardless of how many packages are in the batch.

## Limitations

**Whole-package moves only.** Partial-package moves (moving some but not all files from a directory) are not supported. gopls moves all files in the package together; subsequent files from the same source directory are handled by a filesystem-only step.

**`go.mod` required for cross-directory moves.** The driver reads `go.mod` at the project root to construct the target import path. Without it, cross-directory moves will fail.

**Same-directory moves work without gopls.** Only cross-directory moves need gopls installed. Renames within the same directory are a pure filesystem operation.
