---
name: refac-cli
description: Use when a developer wants to run the `refac` CLI to move or rename files with reference updates. This skill is for using the tool, not for changing the tool's implementation.
---

# Use Refac CLI

`refac` moves or renames source **files** and updates affected import/reference paths. Supported languages: TypeScript, JavaScript, Python, Rust, Go, Dart.

## Hard constraints — read before running

- **Files and directories are both supported for TypeScript/JS.** For all other languages (Python, Rust, Go, Dart), only individual files are supported.
- **One language per call is fine; mixed languages in one call is also fine** — the tool groups them internally.
- `--project-path` must point to the **package root** (the folder that contains `tsconfig.json`, `Cargo.toml`, `pyproject.toml`, etc.), not the monorepo root.
- Paths passed to `--source-path` and `--target-path` may be absolute or relative to `--project-path`.
- Source and target counts must match 1:1. If you have 3 `--source-path` flags you need exactly 3 `--target-path` flags.

## Directory moves (TypeScript/JS only)

Pass the folder path the same way you would a file:

```bash
refac move \
  --project-path /path/to/package \
  --source-path src/old/feature \
  --target-path src/new/feature
```

ts-morph moves all files inside the folder and updates all import paths that reference them, including imports from files **outside** the moved folder.

## Large TypeScript/JS projects — file moves only

For **individual file moves** in projects with more than ~500 TS/JS files, `refac` automatically skips loading the full project and only moves the specific files. This means **cross-project import updates are skipped** in that case.

**Directory moves always load the full project**, regardless of size, because updating external references requires the full context. This may be slow for very large projects.

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
