---
name: refac-cli
description: Use when a developer wants to run the `refac` CLI to move or rename files with reference updates, move TypeScript/JavaScript or Kotlin directories, move a complete Rust module subtree semantically, or rename a symbol (variable, parameter, function, class, method, field) with all its references in TypeScript/JavaScript, Kotlin, Go, Rust, Python, or Dart. Also use it when a rename or move fails because a language server is missing: `refac doctor <language>` explains the fix. For TypeScript/JavaScript, use the package root with its authoritative tsconfig so local callers and aliases can be updated completely. For Kotlin and Android, use the Gradle root. This skill is for using the tool, not changing its implementation.
---

# Use Refac CLI

`refac` moves or renames source files and directories, updating all affected import/reference paths automatically.

## Supported languages

| Language | Files | Directories | Logical modules | Symbols (`rename`) |
|---|---|---|---|---|
| TypeScript / JavaScript | ✅ | ✅ | — | ✅ |
| Kotlin / Android | ✅ | ✅ | — | ✅ |
| Python | ✅ | ❌ | — | ✅ |
| Rust | ✅ | ❌ | ✅ | ✅ |
| Go | ✅ | ❌ | — | ✅ |
| Dart | ✅ | ❌ | — | ✅ |
| Markdown | ✅ | ✅ (folders of Markdown and assets) | — | ❌ |

Passing a directory for any language other than TypeScript/JavaScript, Kotlin and Markdown/asset folders will fail with a clear error.

## Ask the tool

The documentation is inside the binary, in three depths: `refac <command> -h` (a few lines), `refac <command> --help` (everything about one command: rules per language, output, examples) and `refac guide [topic]` (languages, safety, batching, output, servers; `refac guide all` prints every topic). Read `refac guide languages` before the first move or rename in a language you have not used, and `refac guide safety` to know what a failure leaves behind.

## Hard constraints

- For `move`, `--project-path` must be the **package root** (the folder containing `tsconfig.json`, `Cargo.toml`, `go.mod`, etc.) — not the monorepo root.
- For Rust `move-module`, `--project-path` may be a Cargo package or workspace root. Source and target are logical `crate::...` paths in the same crate.
- For TypeScript/JavaScript, ensure `tsconfig.json` includes all local source files. External packages in `node_modules` do not need to be included.
- `--source-path` and `--target-path` must match 1:1. Three sources require three targets.
- Paths may be absolute or relative to `--project-path`.
- Mixed languages in one call are fine — the tool groups them internally.
- TypeScript/JavaScript invocations are limited to 30 contained source files. Directory contents count toward the limit, and the CLI reports the measured count.
- `rename` renames one symbol per call, or several in one call with `--batch` (see below). The language comes from the file extension. `--project-path` is: TypeScript, the package root whose `tsconfig.json` includes every caller (and the tsconfig must be accepted by TypeScript 7: no `baseUrl`, no `moduleResolution: node10`); Kotlin, the Gradle root (`settings.gradle.kts`); Go, the folder with `go.mod` or `go.work`; Rust, the folder with `Cargo.toml`; Python, the folder pyright should treat as the root (it reads `pyrightconfig.json` or `[tool.pyright]` there); Dart, the package folder with `pubspec.yaml` after `dart pub get` (without `.dart_tool/package_config.json` the rename is refused).
- Language servers are never left running: each `move` or `rename` starts the server it needs and stops it afterwards, so do not start one yourself, and there is no memory to give back afterwards. Starting costs seconds (Go and Python 1-8 s, Rust 5-35 s, Kotlin about 40 s and 1.3-1.8 GB of memory while it runs), which is why several renames of one project belong into ONE `rename --batch` call. If a server is not installed, the error lists every place that was looked at and says `Run refac doctor <language>`. Run that command, follow its numbered install steps (or set the environment variable it names), run it again until it prints `ready`, then repeat the original command. `refac doctor` alone shows all languages.
- Kotlin and Android need the JetBrains Kotlin language server (`REFAC_KOTLIN_SERVER`). Each call imports the Gradle build first and takes about 30 to 40 seconds, so put several moves into one `move` call and several renames into one `rename --batch`. A single Kotlin file moved, or a single Kotlin rename, ends with a `// Note:` that shows the batch form. If `REFAC_KOTLIN_BATCH_ONLY=1` is set, a single Kotlin change is refused (dry runs included) with the batch command written out; batch the work, or add `--allow-single` when that one change is all there is. `.java` files, directories with Java sources, and moves between modules or source sets are refused. Read the `// Note:` lines of the output: they list old class names in ProGuard rules, build scripts, and strings that refac does not edit.

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

# see what a move would do first: the paths that move and the edits per file; nothing is written
refac move --dry-run --project-path /path/to/package \
  --source-path src/old.ts --target-path src/new.ts
# (--json adds dry_run, moves, files, edits, notes; Python and Kotlin are planned on a temporary copy)

# batch move (flags in matching order)
refac move \
  --project-path /path/to/package \
  --source-path src/a.ts --source-path src/b.ts \
  --target-path src/x.ts --target-path src/y.ts

# structured output for agent parsing
refac move --json --project-path /path/to/package \
  --source-path src/old.go --target-path pkg/new/old.go

# semantic Rust module move, including its complete physical subtree
refac move-module --project-path /path/to/cargo-workspace \
  crate::engine::matching crate::domain::matching
# add --dry-run first to see the moves and the edits per file; it writes nothing

# rename a TypeScript/JavaScript symbol and every reference to it
refac rename --project-path /path/to/package \
  --file src/lib/util.ts --symbol total --new-name grandTotal

# Kotlin: move a file to another package (package line, imports, Android XML follow)
refac move --project-path /path/to/gradle/root \
  --source-path app/src/main/kotlin/com/example/ui/Home.kt \
  --target-path app/src/main/kotlin/com/example/home/Home.kt

# Kotlin: rename a symbol and every reference
refac rename --project-path /path/to/gradle/root \
  --file app/src/main/kotlin/com/example/util/Helper.kt --symbol shout --new-name yell

# Go, Rust, Python, Dart: the same command, the language follows the extension
refac rename --project-path /path/to/module --file shape/shape.go --symbol Area --new-name Surface
refac rename --project-path /path/to/crate --file src/shapes.rs --symbol area --new-name surface
refac rename --project-path /path/to/project --file shop/shapes.py --symbol area --new-name surface
refac rename --project-path /path/to/package --file lib/shapes.dart --symbol area --new-name surface

# several renames of one project in ONE call (one language, one server start, all or nothing)
refac rename --project-path /path/to/crate --batch renames.json
refac rename --project-path /path/to/crate --batch - --dry-run <<'JSON'
[
  {"file": "src/shapes.rs", "symbol": "SCALE", "new_name": "FACTOR"},
  {"file": "src/shapes.rs", "symbol": "Square", "new_name": "Cube", "line": 36},
  {"file": "src/shapes.rs", "symbol": "FACTOR", "new_name": "MULTIPLIER"}
]
JSON

# a server is missing or broken: this prints what to install and how to check it
refac doctor go
refac doctor                      # all languages at a glance

# preview: plan and verify, change nothing
refac rename --project-path /path/to/package \
  --file src/lib/util.ts --symbol total --new-name grandTotal --dry-run --json

# the name refers to several symbols in the file: choose one by line (and column)
refac rename --project-path /path/to/package \
  --file src/lib/util.ts --symbol total --new-name sumTotal --line 3
```

A rename that would clash with or shadow another symbol, an ambiguous name, an unrenameable symbol, or an unsupported config stops with a message and leaves every file unchanged. Read the message: an ambiguity lists the `--line`/`--column` candidates. After a successful rename read the `Note:` lines: they list where the old name is still written (calls on untyped Python or Dart receivers, strings, comments, Rust `macro_rules!` bodies, which the build needs changed by hand when the note says `ATTENTION`) and end with the `rg -w` command to check them. A package, module, or file name is not a symbol: use `refac move` (or `move-module` for Rust modules).

### Several renames: `--batch`

Use one `--batch` call whenever you have two or more renames in the same project. The language server is started once (not once per rename), every rename is planned, proven and written against the files as the previous one left them, and the server is stopped before the command ends. A later entry may therefore name a symbol by the name an earlier entry gave it, and may name a Kotlin file by the path its class rename moved it to.

- The batch is a JSON list of `{"file", "symbol", "new_name"}` objects, each with an optional `"line"` and `"column"` (same meaning as `--line` and `--column`), read from a file or from stdin with `--batch -`. `--dry-run` and `--json` apply to the whole batch. `--batch` cannot be combined with `--file`, `--symbol`, `--new-name`, `--line` or `--column`.
- All or nothing: if rename 3 of 5 fails, renames 1 and 2 are undone and the message says `Rename 3 of 5 (...) failed; the 2 earlier rename(s) were undone, so nothing was changed`. Fix the entry and run the whole batch again.
- Requests that can be refused beforehand (bad name, keyword, unknown file in the first entry, two projects, two languages, dry-run mixed with real runs) are refused before the server starts.
- One language and one project per batch (TypeScript/JavaScript included). Run one batch per language; a batch that mixes languages is refused before anything runs, and the message names the first entry of the other language.
- A dry-run batch is planned the way the real batch runs: each entry on the files as the entries before it would leave them. So entries that depend on each other (the second names the symbol by the name the first gave it) work in a dry run, and an entry that clashes with an earlier one is refused with the same `Rename N of M` message. A Kotlin dry run of several entries runs the batch on a throw-away copy of the project and takes as long as the real batch. A dry run of several LSP renames stops with a clear message if an entry would move a file, because the next entry could not be planned.

Exit codes: `0` = all succeeded, `1` = one or more failed.

## References

- [Language-specific behaviour](references/language_behaviour.md) — Go whole-package moves, semantic Rust module moves, Dart package config, TS batch memory/watcher behaviour and reference gaps, TS symbol-rename safety rules and limits, Kotlin and Android moves, rename, and refusals, Python re-export limits
- [Install & prerequisites](references/install.md) — build from source, PATH setup, required tooling per language
- [Agent integration](references/agent_integration.md) — how to wire this skill into Claude Code or other agent harnesses via symlink
