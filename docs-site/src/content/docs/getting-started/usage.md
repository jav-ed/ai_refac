---
title: Usage
description: How to use the refac CLI to move files and update references.
---

## Move a single file

```bash
refac move \
  --project-path /path/to/project \
  --source-path src/old/module.ts \
  --target-path src/new/module.ts
```

`--project-path` must be the **package root** — the directory containing `tsconfig.json`, `Cargo.toml`, `go.mod`, `pyproject.toml`, etc. For monorepos, point it at the sub-package being operated on, not the workspace root.

`--source-path` and `--target-path` can be absolute or relative to `--project-path`.

## Avoid repeating the project path

Set `REFAC_PROJECT_PATH` to skip the flag on every call:

```bash
export REFAC_PROJECT_PATH=/path/to/project
refac move --source-path src/old.ts --target-path src/new.ts
```

## Batch move

Repeat the flags in matching order — first source maps to first target:

```bash
refac move \
  --project-path /path/to/project \
  --source-path src/a.ts --source-path src/b.ts \
  --target-path src/x.ts --target-path src/y.ts
```

Mixed languages in one call work — `refac` groups files by language and dispatches each batch to its correct backend independently. If one language's batch fails, the others still run.

## JSON output

```bash
refac move --json \
  --project-path /path/to/project \
  --source-path src/old.go \
  --target-path pkg/new/old.go
```

With `--json`, the response is a single JSON object:

```json
{ "status": "ok", "message": "..." }
```

On partial or full failure, `"status"` is `"error"` and `"message"` contains a structured description of what succeeded and what failed.

## Exit codes

| Code | Meaning |
|---|---|
| `0` | All requested moves completed |
| `1` | One or more moves failed |

## General limitations

- **No dry-run mode.** Changes are applied to disk immediately.
- **No overwrite protection.** If a target path already exists, it will be overwritten.
- Build and vendor directories (`node_modules/`, `target/`, `.git/`, etc.) are excluded from reference scanning.

See the individual [language pages](/languages/typescript/) for language-specific limits.
