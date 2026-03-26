---
name: refac-cli
description: Use when a developer wants to run the `refac` CLI to move or rename files with reference updates. This skill is for using the tool, not for changing the tool's implementation.
---

# Use Refac CLI

`refac` moves or renames source files and directories, updating all affected import/reference paths.

## What is supported per language

| Language | Files | Directories |
|---|---|---|
| TypeScript / JavaScript | ✅ | ✅ |
| Python | ✅ | ❌ |
| Rust | ✅ | ❌ |
| Go | ✅ | ❌ |
| Dart | ✅ | ❌ |
| Markdown | ✅ | ❌ |

**If you pass a directory for a non-TS language, the call will fail.** There is no silent skip — you get an error from the TypeScript driver because it can't find TS source files in that directory.

## Hard constraints — read before running

- `--project-path` must point to the **package root** (the folder that contains `tsconfig.json`, `Cargo.toml`, `pyproject.toml`, etc.), not the monorepo root.
- Paths passed to `--source-path` and `--target-path` may be absolute or relative to `--project-path`.
- Source and target counts must match 1:1. If you have 3 `--source-path` flags you need exactly 3 `--target-path` flags.
- Mixed languages in one call is fine — the tool groups them internally.

## Directory moves (TypeScript/JS only)

Pass the folder path the same way you would a file. ts-morph moves all files inside and rewrites every import that pointed into that folder — including from files **outside** the moved folder.

```bash
refac move \
  --project-path /path/to/package \
  --source-path src/old/feature \
  --target-path src/new/feature
```

Directory moves **always load the full project** regardless of size, because finding external importers requires the full context. This may be slow for very large codebases.

## Large TypeScript/JS projects — file moves only

For individual **file** moves in projects with more than ~500 TS/JS files, `refac` skips loading the full project and only moves the specific files. **Cross-project import updates are skipped in this case.** The file is moved; imports in other files that pointed to it are not updated.

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
