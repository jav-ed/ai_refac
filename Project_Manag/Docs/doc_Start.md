# Doc Start

This repo is a CLI-first refactoring tool. Its job is to move files and update affected references so projects stay consistent after structural changes. The current runtime surface is the `refac` CLI, and the implementation uses language-specific backends for TypeScript/JavaScript, Python, Markdown, Rust, Go, and Dart.

Use this file as the top-level handoff. Do not paste large doc contents into agent context by default. Read only the linked files that are relevant to the task at hand.

Operational decision: after building `refac`, the binary is made available via `~/.local/bin/refac`. During active development, the preferred setup is a symlink from `~/.local/bin/refac` to the release binary. That keeps the command stable while letting rebuilt binaries take effect without any reinstall step.

**Keep the global install current:** every source change requires a `cargo build --release` so the symlinked binary stays in sync with the latest code. If `~/.local/bin/refac` is missing or stale, the globally available command does not reflect recent changes. See [Install & Build](./Guides/dev_guide.md) § 6 for the full workflow.

## Navigation

- **[Capabilities & Language Support](./Descr/abilties.md)** — supported languages and engines, limits per backend, directory moves (TypeScript only), Dart package URI behaviour, JSON output schema
- **[TypeScript / JS](./Features/TypeScript/linker_TypeScript.md)** — tsconfig coverage, low-RAM project loading, aliases, directory moves
- **[Python](./Features/Python/linker_Python.md)** — Rope/Pyrefly backends, re-export limits, namespace packages
- **[Go](./Features/Go/linker_Go.md)** — whole-package moves, batch session architecture, go.mod requirement
- **[Rust](./Features/Rust/linker_Rust.md)** — same-dir vs cross-dir, shim strategy (`#[path]` + `pub use` alias), caller migration
- **[Markdown](./Features/Markdown/linker_Markdown.md)** — relative link rewriting, limits (no code blocks, no absolute URLs)
- **[Install & Build](./Guides/dev_guide.md)** — build from source, symlink to `~/.local/bin/`, cargo install, PATH setup
- **[Testing & Debugging](./Guides/Testing_and_Debugging.md)** — test suite structure, fixture projects, batch move tests, debugging failures
- **[Agent Skill](../../.agents/skills/refac-cli/SKILL.md)** — using `refac` via AI agent, Claude Code integration, language constraints summary
- **[Public Docs Repository](../Setup/internal_Repo_Paths.md)** — how the sibling `Refac_Docs` repository relates to this CLI source of truth
- **[Project Goal](./Descr/goal.md)** — scope, direction, and intended use
- **[Tool Research](./Research/tool_Research_Report.md)** — why each backend was chosen over alternatives
- **[ty / Python Refactoring Notes](./Research/ty_python_refactoring.md)** — why ty is not used for Python moves
