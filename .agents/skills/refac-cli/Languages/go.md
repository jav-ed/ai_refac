# Go

Read before the first Go move or rename, and when one fails. A cross-directory move renames the whole package, which surprises anyone who expects one file to move. A Go call takes 1 to 8 s.

## Setup

- `--project-path`: the folder with `go.mod` or `go.work`.
- Tool: `gopls`. Install it with `go install golang.org/x/tools/gopls@latest`.
- `refac doctor go` finds it (`REFAC_GOPLS` overrides the lookup) and proves it starts.

## Move: whole packages

Moving any `.go` file cross-directory causes gopls to rename the **entire package**. All files in the source directory move together. If `pkg/` contains `a.go`, `b.go`, and `c.go`, asking to move `pkg/a.go` moves all three into the target directory. Partial-package moves are not supported.

Same-directory renames (file rename with no directory change) are a filesystem-only operation, so gopls is not involved and no import paths change.

Requires `go.mod` at the project root for any cross-directory move. Without it the move fails with an error.

## Rename a symbol

`refac rename` renames interface methods together with their implementers and the test variants. Renaming a `package` clause is refused: use `refac move` on the package directory. gopls sometimes answers under load with part of the edits; refac notices and asks again by itself. Several renames of one project go into one `rename --batch` call.
