---
name: refac-cli
description: Use when a developer wants to run the `refac` CLI to move or rename files with reference updates. This skill is for using the tool, not for changing the tool's implementation.
---

# Use Refac CLI

`refac` moves or renames files while updating affected references, and it currently supports TypeScript/JavaScript, Python, Rust, Go, and Dart.

Use `refac move --project-path <project-root> --source-path <old-path> --target-path <new-path>` to move a file, with `--source-path` and `--target-path` resolved relative to `--project-path`.
