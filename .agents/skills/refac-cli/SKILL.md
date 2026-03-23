---
name: refac-cli
description: Use when a developer wants to run the `refac` CLI to move or rename files with reference updates. This skill is for using the tool, not for changing the tool's implementation.
---

# Use Refac CLI

`refac` moves or renames files while updating affected references, and it currently supports TypeScript/JavaScript, Python, Rust, Go, and Dart.

Use it like this when you want to set the project root once via `REFAC_PROJECT_PATH`:

```bash
export REFAC_PROJECT_PATH=/absolute/path/to/project
refac move \
  --source-path src/old_file.ts \
  --target-path src/new_file.ts
```

Use it like this when you want to pass the project root explicitly on the command:

```bash
refac move \
  --project-path /absolute/path/to/project \
  --source-path src/old_file.ts \
  --target-path src/new_file.ts
```

Repeat `--source-path` and `--target-path` in matching order for batch moves.
