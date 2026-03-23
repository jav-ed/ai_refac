---
name: refac-cli
description: Use when a developer wants to run the `refac` CLI to move or rename files with reference updates. This skill is for using the tool, not for changing the tool's implementation.
---

# Use Refac CLI

`refac` moves or renames source **files** and updates affected import/reference paths. Supported languages: TypeScript, JavaScript, Python, Rust, Go, Dart.

## Hard constraints — read before running

- **Only files are supported. Directories are not.** Passing a folder as `--source-path` will error immediately.
- **One language per call is fine; mixed languages in one call is also fine** — the tool groups them internally.
- `--project-path` must point to the **package root** (the folder that contains `tsconfig.json`, `Cargo.toml`, `pyproject.toml`, etc.), not the monorepo root.
- Paths passed to `--source-path` and `--target-path` may be absolute or relative to `--project-path`.
- Source and target counts must match 1:1. If you have 3 `--source-path` flags you need exactly 3 `--target-path` flags.

## Large TypeScript/JS projects

For projects with more than ~500 TS/JS files, `refac` automatically skips loading the full project and only moves the specific files. This means **cross-project import updates are skipped** for large projects — only the moved file itself is written to the new path. Plan accordingly: if reference updates matter, move files one at a time and verify manually, or work in smaller packages.

## Usage

Pass the project root once via env var:

```bash
export REFAC_PROJECT_PATH=/absolute/path/to/package
refac move \
  --source-path src/old_name.ts \
  --target-path src/new_name.ts
```

Or pass it inline:

```bash
refac move \
  --project-path /absolute/path/to/package \
  --source-path src/old_name.ts \
  --target-path src/new_name.ts
```

Batch move (repeat flags in matching order):

```bash
refac move \
  --project-path /absolute/path/to/package \
  --source-path src/a.ts --source-path src/b.ts \
  --target-path src/x.ts --target-path src/y.ts
```

## Help

```bash
refac --help
refac move --help
```
