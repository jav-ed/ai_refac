---
name: refac-cli
description: "Runs the `refac` CLI to move or rename source files, folders and Rust modules with every import and reference updated, and to rename a symbol (variable, parameter, function, class, method, field) everywhere it is used in TypeScript/JavaScript, Kotlin/Android, Go, Rust, Python and Dart. Use when the user asks to move, rename or reorganize files, folders, modules or symbols, mentions refac, or when a move or rename fails because a language server is missing (`refac doctor <language>` explains the fix); it is for using the tool, not for changing its code."
---

# Use Refac CLI

`refac` moves or renames source files and folders and renames symbols, and rewrites every import, reference and path that points at them. Each language is handled by its own tool (language servers, Oxc, Rope), started and stopped by the command itself. Nothing is written until the change is planned and proven, and a failure puts every file back.

The binary carries its own documentation, so read it instead of guessing: `refac <command> -h` (a few lines), `refac <command> --help` (everything about one command: rules per language, output, examples) and `refac guide [topic]` (`languages`, `safety`, `batching`, `output`, `servers`; `refac guide all` prints every topic). Read `refac guide languages` before the first change in a language you have not used, and `refac guide safety` to know what a failure leaves behind.

## Languages

| Language | Files | Directories | Logical modules | Symbols (`rename`) |
|---|---|---|---|---|
| TypeScript / JavaScript | yes | yes | no | yes |
| Kotlin / Android | yes | yes | no | yes |
| Python | yes | no | no | yes |
| Rust | yes | no | yes (`move-module`) | yes |
| Go | yes | no | no | yes |
| Dart | yes | no | no | yes |
| Markdown | yes | yes (folders of Markdown and assets) | no | no |

A directory in any other language fails with a clear error. The language of a file comes from its extension; one `move` call may mix languages.

## The common case

```bash
# move one file; the project path is the root listed in the next section
refac move --project-path /path/to/package \
  --source-path src/old.ts --target-path src/new.ts

# several moves in ONE call: repeat the flags, paired in order
refac move --project-path /path/to/package \
  --source-path src/a.ts --source-path src/b.ts \
  --target-path src/x.ts --target-path src/y.ts

# see the paths that move and the edits per file first; nothing is written
refac move --dry-run --project-path /path/to/package --source-path src/old.ts --target-path src/new.ts

# rename a symbol and every reference (the language follows the file extension)
refac rename --project-path /path/to/package \
  --file src/lib/util.ts --symbol total --new-name grandTotal

# the name refers to several symbols in the file: choose one by line (and column)
refac rename --project-path /path/to/package \
  --file src/lib/util.ts --symbol total --new-name sumTotal --line 3

# several renames of one project in ONE call: one language, one server start, all or nothing
refac rename --project-path /path/to/package --batch - --dry-run <<'JSON'
[
  {"file": "src/lib/util.ts", "symbol": "total", "new_name": "grandTotal"},
  {"file": "src/lib/util.ts", "symbol": "count", "new_name": "size"}
]
JSON

# a whole Rust module with its children, by logical path
refac move-module --project-path /path/to/cargo-workspace crate::engine::matching crate::domain::matching

# a server is missing or broken: prints what to install and how to check it
refac doctor go        # `refac doctor` alone shows every language
```

Add `--json` to `move`, `move-module`, `rename` and `doctor` for one JSON document (an error is JSON on stderr). `REFAC_PROJECT_PATH` sets a default for `--project-path`. Exit code `0` means everything succeeded, `1` means something failed.

## What `--project-path` is

Paths may be absolute or relative to it. It is the root of the project that owns the language's configuration, not the monorepo root.

| Language | `--project-path` |
|---|---|
| TypeScript / JavaScript | package root whose `tsconfig.json` includes every caller; TypeScript 7 must accept it (no `baseUrl`, no `moduleResolution: node10`) |
| Kotlin / Android | Gradle root (`settings.gradle.kts`), not a module folder |
| Go | folder with `go.mod` or `go.work` |
| Rust | folder with `Cargo.toml` (package or workspace) |
| Python | folder pyright treats as the root (`pyrightconfig.json` or `[tool.pyright]`) |
| Dart | package folder with `pubspec.yaml`, after `dart pub get` |

## Rules that change decisions

- `--source-path` and `--target-path` pair one to one in order: three sources need three targets.
- Batch whenever there are two or more changes of one project. Every call starts the server it needs and stops it afterwards (Go and Python 1 to 8 s, Rust 5 to 35 s, Kotlin about 40 s and 1.3 to 1.8 GB), so moves repeat the flags and renames use `--batch`. Never start a server yourself.
- A single Kotlin move or rename is refused by default, dry runs included, before anything starts; the error prints the batch command for that request. When one change really is all there is, add `--allow-single` (or set `REFAC_KOTLIN_BATCH_ONLY=0` for the whole environment).
- TypeScript / JavaScript: at most 30 source files per call (a folder counts the files below it), and only files the tsconfig includes are updated callers.
- A rename that would clash with or shadow another symbol, an ambiguous name, an unrenameable symbol, a symbol that also lives outside the project, or an unsupported config stops with a message and leaves every file unchanged. An ambiguity lists the `--line`/`--column` candidates.
- Read the `// Note:` lines after a success: they list where the old name is still written (strings, comments, untyped Python or Dart receivers, ProGuard rules and build scripts, `macro_rules!` bodies marked `ATTENTION`) and end with an `rg -w` command to check them.
- A package, module or file name is not a symbol: use `move` (`move-module` for Rust modules).
- Server not installed: the error lists every place that was looked at and ends with `Run refac doctor <language>`. Run it, follow its numbered steps (or set the environment variable it names), run it again until it prints `ready`, then repeat the original command.

## Navigation

- [Language-specific behaviour](references/language_behaviour.md): read when a move or rename involves a language with non-obvious semantics or behaves unexpectedly. Go whole-package moves, semantic Rust module moves and what they refuse, Dart package config, TypeScript tsconfig coverage, memory limit and reference gaps, TypeScript symbol-rename safety rules, symbol rename in Go, Rust, Python and Dart, `--batch` behaviour, Kotlin and Android moves, rename and refusals, Python re-export limits, Markdown links.
- [Install and prerequisites](references/install.md): read when `refac` is not installed or not on the PATH, or a language backend (bun, rope, basedpyright, gopls, Dart SDK, Kotlin server) has to be set up.
- [Agent integration](references/agent_integration.md): read only to make an agent harness (Claude Code or another) find this skill, by symlink or copy.
