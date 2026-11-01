---
name: refac-cli
description: "Refactoring CLI (`refac`) that moves or renames files, folders, Rust modules and symbols and updates every import and reference, in TypeScript/JavaScript, Kotlin, Go, Rust, Python, Dart and Markdown. Use when asked to move, rename or reorganize code, when refac is mentioned, or when a move or rename fails on a missing language server."
---

# Use Refac CLI

`refac` moves or renames source files and folders and renames symbols, and rewrites every import, reference and path that points at them, in TypeScript/JavaScript (Astro, MDX), Kotlin/Android, Go, Rust, Python, Dart and Markdown. Each language is handled by its own tool (language servers, Oxc, Rope), started and stopped by the command itself. Nothing is written until the change is planned and proven, and a failure puts every file back.

The binary documents itself, so read it instead of guessing: `refac <command> --help` (rules per language, output, examples) and `refac guide [topic]` (`languages`, `safety`, `batching`, `kotlin`, `output`, `servers`). Read `refac guide safety` to know what a failure leaves behind, and `refac guide batching` before many changes.

## The common case

```bash
# move one file, or several in ONE call (repeat the flags, paired in order)
refac move --project-path /path/to/package --source-path src/old.ts --target-path src/new.ts
refac move --project-path /path/to/package \
  --source-path src/a.ts --source-path src/b.ts --target-path src/x.ts --target-path src/y.ts

# many moves: the list in a file (`--batch -` reads stdin), checked first, then run
echo '[{"source_path": "src/a.ts", "target_path": "src/x.ts"}]' > moves.json
refac move --project-path /path/to/package --batch moves.json --dry-run
refac move --project-path /path/to/package --batch moves.json

# rename a symbol and every reference; add --line (and --column) when the name is ambiguous
refac rename --project-path /path/to/package --file src/lib/util.ts --symbol total --new-name grandTotal

# several renames of one project in ONE call: one language, all or nothing
refac rename --project-path /path/to/package --batch - <<'JSON'
[{"file": "src/lib/util.ts", "symbol": "total", "new_name": "grandTotal"},
 {"file": "src/lib/util.ts", "symbol": "count", "new_name": "size"}]
JSON

# a server is missing or broken: prints what to install and how to check it
refac doctor go
```

`--project-path` is the root of the project that owns the language's configuration (the folder with `tsconfig.json`, `Cargo.toml`, `go.mod` or `pubspec.yaml`, the Gradle root, the pyright root), not the monorepo root; paths may be absolute or relative to it. The language of a file comes from its extension, and one `move` may mix languages. `--dry-run` shows the paths that move and the edits per file and writes nothing. Add `--json` to `move`, `move-module`, `rename` and `doctor` for one JSON document (an error is JSON on stderr). `REFAC_PROJECT_PATH` sets a default for `--project-path`. Exit code `0` means everything succeeded, `1` means something failed.

## Rules that change decisions

- Before the first change in a language, open its file through [Languages](Languages/linker_Languages.md): the exact `--project-path`, limits and rename rules are there. Install a server only for the language of the change at hand: `refac doctor` listing other languages as `MISSING` is expected and needs no action, so never install them to make the list green (the Kotlin server alone is a 1.2 GB download of a build that expires monthly).
- `--source-path` and `--target-path` pair one to one in order: three sources need three targets. `--batch moves.json` takes the same pairs from a file and cannot be combined with the flags.
- Batch whenever there are two or more changes of one project. Every call starts the server it needs and stops it afterwards (seconds, Rust up to 35 s, Kotlin about 24 s and 1.3 to 1.8 GB), so moves repeat the flags or use `--batch moves.json`, and renames use `--batch` (one language and one project, all or nothing). Never start a server yourself: none is kept running between commands, because its view of the files would go stale. A single Kotlin move or rename is refused by default; the Kotlin file says what to do.
- A rename that would clash with or shadow another symbol (the lost or gained usages are listed), an ambiguous name, an unrenameable symbol, a symbol that also lives outside the project, or an unsupported config stops with a message and leaves every file unchanged. An ambiguity lists the `--line`/`--column` candidates.
- Read the `// Note:` lines after a success: they list where the old name is still written (strings, comments, untyped Python or Dart receivers, ProGuard rules and build scripts, `macro_rules!` bodies marked `ATTENTION`) and end with an `rg -w` command to check them.
- A package, module or file name is not a symbol: use `move` (`move-module` for a Rust module).
- Server not installed for this change's language: the error lists every place that was looked at and ends with `Run refac doctor <language>`. Run it, follow its numbered steps (or set the environment variable it names), run it again until it prints `ready`, then repeat the original command.

## Navigation

- [Languages](Languages/linker_Languages.md): what each language moves and renames, its `--project-path`, and one file per language (TypeScript/JavaScript/Astro/MDX, Kotlin/Android, Rust, Go, Python, Dart, Markdown) with its setup, limits and rename rules. Open it before the first change in a language.
- [Install refac](Setup/install.md): read when `refac` itself is not installed or not on the PATH. Build from source, the PATH link, the platform, and where each language's tools are described.
- [Agent integration](Setup/agent_Integration.md): read only to make an agent harness (Claude Code or another) find this skill, by symlink or copy.
