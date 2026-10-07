# Doc Start

*This `doc_Start.md` is the docs entry point, structured so an agent can quickly decide what to read and what to skip. It opens with a short summary of the repo and key entry-point files, then routes to each topic area through labeled links. Open a linker only when the task calls for it; the labels are written to make that decision possible without clicking.*

This repo is a CLI-first refactoring tool. Its job is to move files and update affected references so projects stay consistent after structural changes, and to rename TypeScript/JavaScript and Kotlin symbols with every reference. The current runtime surface is the `refac` CLI, and the implementation uses language-specific backends for TypeScript/JavaScript, Python, Markdown, Rust, Go, Dart, and Kotlin (JVM and Android).

Operational decision: after building `refac`, the binary is made available via `~/.local/bin/refac`. During active development, the preferred setup is a symlink from `~/.local/bin/refac` to the release binary. That keeps the command stable while letting rebuilt binaries take effect without any reinstall step.

**Keep the global install current:** every source change requires a `cargo build --release` so the symlinked binary stays in sync with the latest code. If `~/.local/bin/refac` is missing or stale, the globally available command does not reflect recent changes. See [Install & Build](Guides/dev_guide.md) § 6 for the full workflow.

Entry points: `src/bin/refac.rs` and `src/cli.rs` (commands `move`, `move-module`, `rename`), `src/logic/mod.rs` (move dispatch by language), `src/logic/route.rs` (which language a path belongs to), `src/drivers/` (one backend per language), `scripts/ts_refactor.ts` (TypeScript move helper run by Bun).

## Docs

- [Capabilities & Language Support](Descr/abilties.md): supported languages and engines, the limits of each backend, directory moves (TypeScript and Kotlin), symbol rename (TypeScript and Kotlin), Dart package URI behaviour, and the JSON output schema.
- [TypeScript / JS](Features/TypeScript/linker_TypeScript.md): file and directory moves (Oxc parsing, TypeScript resolution, rollback, memory limits, aliases) and `refac rename` for symbols (TypeScript 7 native engine, in-memory verification, hard-fail rules, the legacy `baseUrl` limit).
- [Kotlin / Android](Features/Kotlin/linker_Kotlin.md): file and package moves, `refac rename` for symbols, and the Android layer (manifest and layout XML, `R` and `BuildConfig`) on top of the JetBrains Kotlin language server; what is refused, what is reported, and where each part of the code lives.
- [Python](Features/Python/linker_Python.md): Rope and Pyrefly backends, re-export limits, namespace packages.
- [Go](Features/Go/linker_Go.md): whole-package moves, batch session architecture, the `go.mod` requirement.
- [Rust](Features/Rust/linker_Rust.md): LSP file renames, semantic module-subtree moves, workspace reference migration, strict v1 limits.
- [Markdown](Features/Markdown/linker_Markdown.md): moving Markdown files, assets, and document folders with every link that follows (inline, reference, HTML, `%20`, queries), fixing Markdown links to files other backends moved, the rollback and refusal rules, and the limits (wiki-links, MDX imports, front matter).
- [Install & Build](Guides/dev_guide.md): build from source, symlink to `~/.local/bin/`, cargo install, PATH setup.
- [Testing & Debugging](Guides/Testing_and_Debugging.md): test suite structure, fixture projects, batch move tests, rename tests, the ignored real-server Kotlin tests, debugging failures.
- [Agent Skill](../../.agents/skills/refac-cli/SKILL.md): using `refac` through an AI agent, Claude Code integration, language constraints summary.
- [Project Goal](Descr/goal.md): scope, direction, and intended use.
- [Investigations](Investigation/linker_Investigation.md): evidence behind engine choices: the TypeScript symbol-rename engine comparison with test cases, speed, memory, and the silent failures found, and the Kotlin tool comparison with the hands-on results against the real server.
- [Tool Research](Research/tool_Research_Report.md): why each file-move backend was chosen over alternatives.
- [ty / Python Refactoring Notes](Research/ty_python_refactoring.md): why ty is not used for Python moves.
- [Open issues](../Live_Working/open_Issues.md): active items such as the logo, docs publishing, and the skill-file onboarding idea.

## Repo References

- [Internal repo paths](Setup/internal_Repo_Paths.md): the sibling `Refac_Docs` repository (public Fumadocs site) and how it relates to this CLI source of truth.
- [External reference repos](Setup/repos_List.md): upstream source clones kept under the gitignored `Repos/` folder, with the commands to restore them.
- [Tool versions](Setup/tool_Versions.md): every tool and library refac builds on or drives (Rust toolchain, `ra_ap_*` pins, Bun and npm packages, language servers, Gradle/AGP/Kotlin fixtures), the version verified, where it is pinned, and what constrains a bump. Open it before updating anything.
- [Kotlin server setup](Setup/kotlin_Server.md): downloading and verifying the JetBrains Kotlin language server, `REFAC_KOTLIN_SERVER`, the timeout variable, measured startup time and memory, and how to run the real-server tests.
- [Handoff](Setup/handoff_Continuation.md): where the work stands (TypeScript rename and the Kotlin backend done), what is not in git, what is untested, and the first commands to run.

Note: this file lives at `Project_Manag/Docs/doc_Start.md`, so all link paths above are relative to `Project_Manag/Docs/`.
