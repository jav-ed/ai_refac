---
name: refac-cli
description: Use when a developer wants to run the `refac` CLI to move or rename files with reference updates. This skill is for using the tool, not for changing the tool's implementation.
---

# Use Refac CLI

`refac` moves or renames source files and directories, updating all affected import/reference paths automatically.

## Supported languages

| Language | Files | Directories |
|---|---|---|
| TypeScript / JavaScript | ✅ | ✅ |
| Python | ✅ | ❌ |
| Rust | ✅ | ❌ |
| Go | ✅ | ❌ |
| Dart | ✅ | ❌ |
| Markdown | ✅ | ❌ |

Passing a directory for any non-TS/JS language will fail with a clear error.

## Hard constraints

- `--project-path` must be the **package root** (the folder containing `tsconfig.json`, `Cargo.toml`, `go.mod`, etc.) — not the monorepo root.
- `--source-path` and `--target-path` must match 1:1. Three sources require three targets.
- Paths may be absolute or relative to `--project-path`.
- Mixed languages in one call are fine — the tool groups them internally.

## Usage

```bash
# single file
refac move \
  --project-path /path/to/package \
  --source-path src/old.ts \
  --target-path src/new.ts

# set project path once via env var
export REFAC_PROJECT_PATH=/path/to/package
refac move --source-path src/old.ts --target-path src/new.ts

# batch move (flags in matching order)
refac move \
  --project-path /path/to/package \
  --source-path src/a.ts --source-path src/b.ts \
  --target-path src/x.ts --target-path src/y.ts

# structured output for agent parsing
refac move --json --project-path /path/to/package \
  --source-path src/old.go --target-path pkg/new/old.go
```

Exit codes: `0` = all succeeded, `1` = one or more failed.

## References

- [Language-specific behaviour](references/language_behaviour.md) — Go whole-package moves, Rust shim strategy, Dart package config, TS large-project threshold, Python re-export limits
- [Install & prerequisites](references/install.md) — build from source, PATH setup, required tooling per language
- [Agent integration](references/agent_integration.md) — how to wire this skill into Claude Code or other agent harnesses via symlink
