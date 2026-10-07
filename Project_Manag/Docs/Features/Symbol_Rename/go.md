# Go symbol rename

`refac rename` on a `.go` file renames a variable, constant, function, method, struct field, type, or interface method with gopls, across every package of the module. Go's package-per-folder model needs nothing extra: gopls knows the whole module.

```bash
refac rename --project-path /path/to/module --file shape/shape.go \
  --symbol Area --new-name Surface
```

`--project-path` must hold `go.mod` or `go.work`; without either the command stops and says so. gopls is found by [the locator](../../Setup/language_Servers.md) (`REFAC_GOPLS`, `PATH`, `$GOBIN`, `$GOPATH/bin`, `~/go/bin`); if it is missing the error names `refac doctor go`, which prints `go install golang.org/x/tools/gopls@latest`.

## What gopls does by itself

- **Refuses what breaks the build.** A new name that collides in scope is refused ("already declared"), and so is a method rename that would stop a type implementing its interface. The reason comes back as the error, and nothing is written.
- **Renames interface families.** Renaming an interface method renames the methods that implement it, in other packages too. The edits at those methods lie outside the references of the named symbol, so the engine treats each implementer as a related group and proves it separately ([Engine](engine.md)).
- **Renames test variants.** Files such as `shape_test.go` in the same or an external test package are edited.
- **Edits doc comments** that start with the symbol name. Those edits sit outside every reference, so the engine accepts an edit on a `//` comment line that replaces exactly the old name and nothing else.

## What refac adds

- **Asks again.** Under CPU load gopls answers about one rename in four with only part of the edits (the implementing methods and the test variant missing) while its references stay complete. Refac notices that a listed reference was not edited and asks again, up to four times, with the documents put back first. The measured symptom without this was a verification failure that disappeared on a quiet machine; with it the nine real-server tests passed 16 of 16 runs under load.
- **Refuses a package rename.** Renaming the `package` clause makes gopls answer with file moves and deletes. The command stops with "use `refac move` with the package directory", which does the whole-package move with its importers.
- **Reports what is still written.** After the plan, the sources are scanned for the old name as a whole word: other symbols with the same name, strings, comments. The note ends with the `rg -w` command to check them.

## Limits

- gopls loads one build configuration. Files excluded by build constraints for this platform are outside its view; the leftover note lists any that still spell the old name.
- Generated code is renamed only where gopls sees it; regenerate afterwards.
- Reflection, struct tags, and `go:generate` arguments are text, not symbols; the leftover note shows them.

## Tests

`tests/go_rename.rs` (nine scenarios, `#[ignore]`d, fixture `tests/fixtures/go/rename_module`): an interface method with implementers and callers, a method that would stop implementing its interface (refused), a function used across packages, a variable and a struct field across packages, a name shared by several symbols (listed, then chosen by line), a collision (refused, nothing changes), a package rename (refused, points to `refac move`), a dry run, and text that is not a symbol. Each successful scenario ends with `go build ./...` and `go vet ./...`. Run with gopls installed: `cargo test --test go_rename -- --ignored`.
