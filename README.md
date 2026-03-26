# refac

A CLI tool that moves source files and updates affected import paths, module references, and links across a project. Designed for scripted and agent-driven workflows where an IDE is not in the loop.

> Built for personal use. If it's useful to you, go ahead — no guarantees.

---

## Supported languages

| Language | Files | Directories | Engine |
|---|---|---|---|
| TypeScript / JavaScript | ✅ | ✅ | ts-morph via Bun |
| Python | ✅ | ❌ | Rope (automatic fallback: Pyrefly) |
| Rust | ✅ | ❌ | rust-analyzer (LSP) |
| Go | ✅ | ❌ | gopls (LSP) |
| Dart | ✅ | ❌ | Dart analysis server (LSP) |
| Markdown | ✅ | ❌ | Native (no external tooling) |

Language is detected by file extension (`.ts`, `.tsx`, `.js`, `.jsx`, `.py`, `.rs`, `.go`, `.dart`, `.md`). Directory sources are routed to the TypeScript driver; all other languages require individual files.

---

## Install

**Requires Rust 1.85+ (edition 2024).** Install via [rustup](https://rustup.rs) if needed.

```bash
git clone <repo-url>
cd refac
cargo build --release
```

Add the binary to your PATH. From inside the repo directory:

```bash
# symlink — rebuilding updates it automatically
ln -sf "$(pwd)/target/release/refac" ~/.local/bin/refac

# or copy a fixed snapshot
cp target/release/refac ~/.local/bin/refac

# or install from the local checkout
cargo install --path .
```

**Platform:** Linux and macOS. Windows is untested and not supported.

Each language backend requires its own tooling — see [Prerequisites](#prerequisites).

---

## Usage

### Move a single file

```bash
refac move \
  --project-path /path/to/project \
  --source-path src/old/module.ts \
  --target-path src/new/module.ts
```

`--project-path` must be the **package root** — the directory that contains `tsconfig.json`, `Cargo.toml`, `go.mod`, `pyproject.toml`, etc. For monorepos or workspaces, point it at the sub-package being operated on, not the workspace root.

Paths given to `--source-path` and `--target-path` can be absolute or relative to `--project-path`.

Set `REFAC_PROJECT_PATH` to avoid repeating it:

```bash
export REFAC_PROJECT_PATH=/path/to/project
refac move --source-path src/old.ts --target-path src/new.ts
```

### Batch move

Repeat the flags in matching order — first source maps to first target, and so on:

```bash
refac move \
  --project-path /path/to/project \
  --source-path src/a.ts --source-path src/b.ts \
  --target-path src/x.ts --target-path src/y.ts
```

Mixed languages in one call work — the tool groups files by language and dispatches each batch to its correct backend. If one language's batch fails, the others still run. The response reports which succeeded and which failed.

### JSON output

```bash
refac move --json \
  --project-path /path/to/project \
  --source-path src/old.go \
  --target-path pkg/new/old.go
```

With `--json`, the response is a single JSON object:

```json
{
  "status": "ok",
  "message": "..."
}
```

On partial or full failure, `"status"` is `"error"` and `"message"` contains a structured description of what succeeded and what failed.

### Exit codes

- `0` — all requested moves completed
- `1` — one or more moves failed (or all failed)

---

## Limitations

These are not edge cases. Read them before deciding whether this tool is right for your situation.

**TypeScript / JavaScript**
- File moves in projects with more than ~500 source files skip cross-project import updates. Only the moved file's own imports are rewritten; nothing that imports it is updated. Point `--project-path` at a sub-package root (not the monorepo root) to stay under the threshold.
- Directory moves always load the full project and may be slow on large codebases.
- The 500-file threshold excludes `node_modules`, `dist`, `build`, `.next`, and `.git`.

**Python**
- Rope cannot trace imports that go through `__init__.py` re-exports. If a package re-exports a symbol and callers import via the re-export, those callers are not updated.
- Namespace packages (no `__init__.py`) may see incomplete updates.

**Rust**
- **Cross-directory moves do not rewrite caller imports.** Moving a file to a different directory adds a `#[path = "..."]` attribute in the declaring file and a `pub use crate::...` alias in the target module. These are permanent code changes that will appear in your diff. Existing caller files continue to compile through the alias but their import paths are not migrated. Same-directory renames do fully rewrite all `use` paths via rust-analyzer.
- Single crate only. Cross-crate reference updates are not supported.

**Go**
- **Moving any `.go` file cross-directory renames the entire package.** All files in the source directory move together. If `pkg/` contains `a.go`, `b.go`, and `c.go`, asking to move `pkg/a.go` will cause gopls to move all three. Partial-package moves are not supported.
- Same-directory moves (rename only, no package change) are a filesystem-only operation — gopls is not involved and no import paths change.
- Requires `go.mod` at the project root for cross-directory moves.

**Dart**
- `package:` URI imports are only rewritten if `.dart_tool/package_config.json` exists at the project root. Run `dart pub get` to generate it. Without it, only relative imports are updated.

**Markdown**
- Only relative links are rewritten. Absolute URLs and `http://` / `https://` links are left unchanged.
- Links inside fenced code blocks and inline code spans are not rewritten.

**General**
- No dry-run mode. Changes are applied to disk immediately.
- If a target path already exists, the tool will overwrite it.
- The tool does not walk into `node_modules/`, `target/`, `.git/`, or similar build/vendor directories when scanning for references.

---

## Prerequisites

| Language | Required | Install |
|---|---|---|
| TypeScript / JS | `bun` | [bun.sh](https://bun.sh) |
| Python | `rope` importable from `.venv` or `python3` | `pip install rope` |
| Python (fallback) | `pyrefly` (only if Rope is absent) | `pip install pyrefly` |
| Rust | `rust-analyzer` | [rust-analyzer.github.io](https://rust-analyzer.github.io) |
| Go | `gopls` | `go install golang.org/x/tools/gopls@latest` |
| Dart | Dart SDK | [dart.dev/get-dart](https://dart.dev/get-dart) |
| Markdown | none | — |

No specific minimum version is enforced for any external tool, but use recent releases. Older versions of gopls and rust-analyzer may behave differently or not at all.

---

## How it works

The approach depends on the language:

**LSP-backed (Rust, Go, Dart):** The tool starts a language server process, issues a rename request (`textDocument/rename` or `workspace/willRenameFiles`), applies the workspace edit the server returns, then moves the file on the filesystem. For batch operations, multiple renames are sent within a single server session with `textDocument/didChange` notifications between them to keep the server's view current.

**ts-morph (TypeScript / JavaScript):** A Bun script loads the project using ts-morph (a TypeScript Compiler API wrapper), performs the move, and ts-morph rewrites all affected import paths using the compiler's own reference graph.

**Rope (Python):** The Rope refactoring library is invoked directly via Python. It performs the move and updates all import statements it can trace.

**Native (Markdown):** The tool parses Markdown link syntax directly in Rust, computes new relative paths, and rewrites affected links. No external tooling required.

---

## Running tests

```bash
cargo test
```

The suite covers unit tests and integration tests for all supported languages. Integration tests copy fixture projects into temp directories and run assertions on the resulting files. Tests that require external tools (gopls, rust-analyzer, etc.) skip gracefully if the tool is not installed — they do not fail, but they also do not provide coverage.

---

## Contributing

Issues and pull requests are welcome. There is no formal contribution guide yet — open an issue first if you are planning a significant change.

---

## License

Hippocratic License HL3 — see [LICENSE](LICENSE).
