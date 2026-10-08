# refac

A CLI tool that moves source files and updates affected import paths, module references, and links across a project, and renames symbols (variables, parameters, functions, methods, fields, types) with every reference in TypeScript/JavaScript, Kotlin, Go, Rust, Python, and Dart. Designed for scripted and agent-driven workflows where an IDE is not in the loop.

> Built for personal use. If it's useful to you, go ahead — no guarantees.

Public user documentation is maintained independently in [`jav-ed/refac-docs`](https://github.com/jav-ed/refac-docs). This repository remains the source of truth for CLI behavior and limitations.

---

## The story

I was building out an AI agent workflow and kept running into the same problem: when an agent needs to move a file, it does it the hard way — reads every file that imports it, rewrites the paths manually, hopes it got them all. That is slow, token-heavy, and fragile. A missed import means a broken build, a retry, more context burned.

Moving files and updating references is exactly what refactoring tools are built for. The agent should call the right tool, get a clean result, and move on.

`refac` is that tool. Three things make it agent-friendly by design:

**1. Ships with a skill file — no MCP needed.**
The repo includes a `.agents/skills/refac-cli/` folder. Drop it into your agent setup and it loads only when relevant. The agent reads what it needs, skips the rest. No persistent context overhead, no server to run.

**2. Structured output an agent can actually use.**
The `--json` flag returns a predictable JSON object with `status`, `operation`, and operation-specific fields, so the agent can parse the result cleanly without scraping terminal output. Errors return `status` and a descriptive `error`.

**3. Built-in help in three depths, for agents and humans alike.**
Everything is inside the binary, so it is there when this repository is not.

```bash
refac move -h          # a few lines: what the command is and its flags
refac move --help      # everything about one command: arguments, rules per language, output, examples
refac guide            # in-depth topics across commands: languages, safety, batching, output, servers
refac guide languages  # one topic (refac guide all prints every topic)
```

`refac --help` is the whole tool on one page: commands, what each language supports, how a change stays safe, what a command costs, exit codes and environment variables. `refac doctor` checks the language servers and prints the install steps for a missing one.

---

## Ask your agent

The docs in this repo are structured for agent navigation, not for sequential reading. Instead of skimming through files yourself, point your agent at the entry point and ask your question directly:

```
Read @Project_Manag/Docs/doc_Start.md and tell me: [your question here]
```

Examples:

```
Read @Project_Manag/Docs/doc_Start.md and tell me how to install this tool.
Read @Project_Manag/Docs/doc_Start.md and explain how Go package moves work.
Read @Project_Manag/Docs/doc_Start.md and tell me what languages are supported and what their limits are.
Read @Project_Manag/Docs/doc_Start.md and explain semantic Rust module moves.
```

The agent will navigate to the relevant doc, read only what it needs, and answer directly.

---

## Agent integration

The skill file lives in `.agents/skills/refac-cli/` — this is the single source of truth. To wire it into your agent tool, symlink from wherever that tool expects skills rather than copying.

### Claude Code

```bash
mkdir -p .claude
ln -s ../.agents/skills .claude/skills
```

Claude Code picks up skills from `.claude/skills/`. The symlink points back to `.agents/skills/`, so there is no duplication — one folder, two entry points.

The `.claude/` directory is tracked in git. `.claude/skills` is listed in `.gitignore` so the symlink itself is not committed (the skills are already tracked under `.agents/`).

Other agent tools that support a skills or prompts directory can be wired up the same way — just symlink from their expected path into `.agents/skills/`.

---

## Supported languages

| Language | Files | Directories | Logical modules | Engine |
|---|---|---|---|---|
| TypeScript / JavaScript | ✅ | ✅ | — | Oxc parser + TypeScript resolver via Bun; symbol rename via the TypeScript 7 native language server |
| Python | ✅ | ❌ | — | Rope (automatic fallback: Pyrefly); symbol rename via basedpyright |
| Rust | ✅ | ❌ | ✅ | rust-analyzer LSP + embedded HIR; symbol rename via rust-analyzer |
| Go | ✅ | ❌ | — | gopls (LSP), also for symbol rename |
| Dart | ✅ | ❌ | — | Dart analysis server (LSP), also for symbol rename |
| Markdown | ✅ | ❌ | — | Native (no external tooling) |
| Kotlin / Android | ✅ | ✅ | — | JetBrains Kotlin language server, plus refac's own Android XML layer |

Symbol rename (`refac rename`) is available for TypeScript / JavaScript, Kotlin, Go, Rust, Python, and Dart. Language servers are started for the command and stopped afterwards, never left running. If one is not installed, the error lists where it looked and names `refac doctor <language>`, which prints the install steps; `refac doctor` alone shows every language.

Language is detected by file extension (`.ts`, `.tsx`, `.js`, `.jsx`, `.mts`, `.cts`, `.mjs`, `.cjs`, `.py`, `.rs`, `.go`, `.dart`, `.kt`, `.md`). A directory source is routed by what it contains: TypeScript/JavaScript files, or Kotlin files anywhere below it; all other languages require individual files.

---

## Install

**Requires Rust 1.99+ (edition 2024).** The repository pins Rust 1.99.0, rustfmt, and rust-analyzer in `rust-toolchain.toml`. Install via [rustup](https://rustup.rs) if needed.

```bash
git clone https://github.com/jav-ed/ai_refac.git
cd ai_refac
cargo build --release
```

Add the binary to your PATH. From inside the repo directory:

```bash
# symlink — rebuilding updates it automatically
ln -sf "$(pwd)/target/release/refac" ~/.local/bin/refac

# or copy a fixed snapshot
cp target/release/refac ~/.local/bin/refac

# or install from the local checkout
cargo install --path .
```

The TypeScript and Python helpers (`scripts/`) stay in the checkout: refac finds them next to the binary (`target/release`), or in the checkout it was built from, so keep the checkout where you built it. When it cannot find one, the error lists every place it looked.

**Platform:** Linux and macOS. Windows is untested and not supported.

Each language backend requires its own tooling — see [Prerequisites](#prerequisites).

---

## Usage

### Move a single file

```bash
refac move \
  --project-path /path/to/project \
  --source-path src/old/module.ts \
  --target-path src/new/module.ts
```

For `move`, `--project-path` must be the **package root** — the directory that contains `tsconfig.json`, `Cargo.toml`, `go.mod`, `pyproject.toml`, etc. For monorepos or workspaces, point it at the sub-package being operated on, not the workspace root.

Paths given to `--source-path` and `--target-path` can be absolute or relative to `--project-path`.

Set `REFAC_PROJECT_PATH` to avoid repeating it:

```bash
export REFAC_PROJECT_PATH=/path/to/project
refac move --source-path src/old.ts --target-path src/new.ts
```

### Batch move

Repeat the flags in matching order — first source maps to first target, and so on:

```bash
refac move \
  --project-path /path/to/project \
  --source-path src/a.ts --source-path src/b.ts \
  --target-path src/x.ts --target-path src/y.ts
```

Mixed languages in one call work — the tool groups files by language and dispatches each batch to its correct backend. If one language's batch fails, the others still run. The response reports which succeeded and which failed.

### Move a Rust module subtree

Use logical module paths, not filesystem paths, for a structural Rust move:

```bash
refac move-module \
  --project-path /path/to/cargo-workspace \
  crate::engine::matching \
  crate::domain::matching
```

`--project-path` may be a Cargo package or workspace root. Refac resolves the source module semantically, moves its complete conventional file/`mod.rs` subtree, rewrites resolved references in the workspace, adjusts affected `super::` paths, creates missing parent modules, and validates the result with `cargo check --workspace --all-targets`.

### Rename a TypeScript / JavaScript symbol

```bash
refac rename \
  --project-path /path/to/package \
  --file src/lib/util.ts \
  --symbol total \
  --new-name grandTotal
```

`--project-path` is the package root whose `tsconfig.json` includes every caller. Refac finds every reference with the TypeScript 7 native language server (imports, aliases, re-exports, namespace access, class members, JSX, `.js` files), plans all edits, proves in memory that the new name neither clashes with nor shadows another symbol, and only then writes. Use `--dry-run` to preview the edits. If the name refers to several symbols in the file, the command lists them; pass `--line` (and `--column`) to choose one. Strings and comments are never edited. See [Symbol rename](Project_Manag/Docs/Features/TypeScript/symbol_Rename.md) for the safety rules and limits.

### Move or rename in Kotlin and Android

```bash
refac move \
  --project-path /path/to/gradle/root \
  --source-path app/src/main/kotlin/com/example/ui/Home.kt \
  --target-path app/src/main/kotlin/com/example/home/Home.kt

refac rename \
  --project-path /path/to/gradle/root \
  --file app/src/main/kotlin/com/example/util/Helper.kt \
  --symbol shout --new-name yell
```

`--project-path` is the Gradle root (the folder with `settings.gradle.kts`). The JetBrains Kotlin language server rewrites the `package` line, imports, and usages, including Java callers, and renames symbols with every reference. For Android, refac adds what the server never touches: class names in the manifest, layouts, and navigation graphs, and the `R` / `BuildConfig` imports a file loses when it leaves its namespace package. A rename that would clash with or shadow another declaration is refused, and every failure restores the project. Old names in ProGuard rules, build scripts, and strings are reported, not edited. The server is installed once by you; see [Kotlin server setup](Project_Manag/Docs/Setup/kotlin_Server.md) and [Kotlin and Android](Project_Manag/Docs/Features/Kotlin/linker_Kotlin.md).

### Rename a symbol in Go, Rust, Python, or Dart

```bash
refac rename --project-path /path/to/module  --file shape/shape.go    --symbol Area --new-name Surface
refac rename --project-path /path/to/crate   --file src/shapes.rs     --symbol area --new-name surface
refac rename --project-path /path/to/project --file shop/shapes.py    --symbol area --new-name surface
refac rename --project-path /path/to/package --file lib/shapes.dart   --symbol area --new-name surface
```

`--project-path` is the folder with `go.mod`, `Cargo.toml`, or `pubspec.yaml` (after `dart pub get`); for Python it is the folder basedpyright treats as the root. The same rules hold in all four: the server (gopls, rust-analyzer, basedpyright, the Dart SDK's analysis server) lists the references and proposes the edits, refac proves in memory that the renamed program still means the same (a new name that clashes with or shadows something is refused with the usages that would change), writes through an undo log, and reports where the old name is still written (untyped receivers, strings, comments, Rust `macro_rules!` bodies). `--dry-run` writes nothing; `--line` and `--column` choose one symbol when the name refers to several. Package, module, and file names are paths: use `refac move` or `move-module`. Details per language: [Symbol rename](Project_Manag/Docs/Features/Symbol_Rename/linker_Symbol_Rename.md).

**Several renames in one call.** Starting a language server costs seconds (Kotlin about 40 s), so several renames of one project go into one `--batch` call: the server starts once, each rename is proven and written against the files the previous one left, and the batch is all or nothing (a failing rename undoes the earlier ones). The batch is a JSON list read from a file or from stdin with `-`; it works for Kotlin, Go, Rust, Python, and Dart, one language and one project per batch.

```bash
refac rename --project-path /path/to/crate --batch - <<'JSON'
[
  {"file": "src/shapes.rs", "symbol": "SCALE", "new_name": "FACTOR"},
  {"file": "src/shapes.rs", "symbol": "Square", "new_name": "Cube", "line": 36},
  {"file": "src/shapes.rs", "symbol": "FACTOR", "new_name": "MULTIPLIER"}
]
JSON
```

On the Rust fixture four renames took 18.8 s as four commands and 4.6 s as one batch. No server stays running between commands, by design: a resident server would hold hundreds of MB to a few GB of memory and would see a stale copy of every file that anything else edits.

### Fix a missing language server

```bash
refac doctor            # every language: ready, MISSING, or BROKEN
refac doctor go         # where gopls was looked for, the install command, a start check
```

### JSON output

```bash
refac move --json \
  --project-path /path/to/project \
  --source-path src/old.go \
  --target-path pkg/new/old.go
```

With `--json`, a file move returns a single JSON object with operation-specific fields:

```json
{
  "status": "ok",
  "operation": "move",
  "project_path": "/path/to/project",
  "source_path": ["src/old.go"],
  "target_path": ["pkg/new/old.go"],
  "result": "..."
}
```

On failure, `"status"` is `"error"` and `"error"` contains the descriptive failure chain. Successful `move-module --json` output additionally reports `source_module`, `target_module`, `dry_run`, `moved_paths`, `edited_files`, `edits`, `files` (path and number of edits) and `moves` (from, to). `move-module --dry-run` plans the move, lists what it would move and edit, and writes nothing; the Cargo check that proves the result compiles runs only on a real move.

### Exit codes

- `0` — all requested moves completed
- `1` — one or more moves failed (or all failed)

---

## Limitations

These are not edge cases. Read them before deciding whether this tool is right for your situation.

**TypeScript / JavaScript**
- Point `--project-path` at the package with the authoritative `tsconfig.json`; its `include` or `files` configuration must cover all local TS/JS sources. External packages in `node_modules` do not need to be included.
- Without tsconfig, Refac globs local source files but alias and module resolution is weaker.
- Each invocation is limited to 30 contained TS/JS source files, including files inside requested directories.

**Python**
- Rope cannot trace imports that go through `__init__.py` re-exports. If a package re-exports a symbol and callers import via the re-export, those callers are not updated.
- Namespace packages (no `__init__.py`) may see incomplete updates.

**Rust**
- Use `move-module` for cross-directory or otherwise structural moves. Ordinary `move` rejects cross-directory `.rs` paths instead of guessing the logical module change.
- Source and target must be logical `crate::...` paths in the same crate. Workspace dependants are updated, but a source path that resolves in multiple workspace crates is rejected as ambiguous.
- v1 rejects inline source modules, `#[path]` and `#[macro_use]` declarations (other attributes must be conditions or lints such as `#[cfg]`), `pub(in …)` visibility that is not an absolute `crate::` path, syntax errors, and complex paths it cannot preserve. It handles code behind `#[cfg(test)]`, `crate::` paths in macro arguments, short references through `use`, `pub(super)`, and a module that leaves a grouped import (details and limits: [Rust](Project_Manag/Docs/Features/Rust/linker_Rust.md)).
- The semantic command never adds `#[path]` or compatibility re-export shims. It validates after applying and rolls planned source changes back on failure.

**Go**
- **Moving any `.go` file cross-directory renames the entire package.** All files in the source directory move together. If `pkg/` contains `a.go`, `b.go`, and `c.go`, asking to move `pkg/a.go` will cause gopls to move all three. Partial-package moves are not supported.
- Same-directory moves (rename only, no package change) are a filesystem-only operation — gopls is not involved and no import paths change.
- Requires `go.mod` at the project root for cross-directory moves.

**Dart**
- `package:` URI imports are only rewritten if `.dart_tool/package_config.json` exists at the project root. Run `dart pub get` to generate it. Without it, a move that would leave a `package:` import pointing at a file that no longer exists is refused before anything is written, and the error lists the imports.

**Kotlin / Android**
- Every call starts the Kotlin language server and imports the Gradle build first: about 30 seconds and about 1.6 GiB of memory for the server on a tiny project. Batch several moves into one call.
- `.java` files, directories containing Java sources, and moves between modules or source sets are refused. Kotlin Multiplatform is not supported.
- Old class names in ProGuard rules, build scripts, service lists, and string literals are reported, not rewritten.

**Markdown**
- Only relative file paths are rewritten. Web addresses, `#anchors`, site-root `/paths`, and paths with template syntax are left unchanged.
- The file is read as CommonMark, so links inside code (fenced blocks, indented blocks, inline spans), HTML comments, and front matter are not rewritten. HTML `href`, `src`, `poster`, and `srcset` values are rewritten; MDX imports, wiki-links `[[Page]]`, and paths inside other file types (SVG, CSS, JSON) are not.
- Files ignored by the project's `.gitignore`, `.git/`, and `node_modules/` are not searched. Markdown files that are not valid UTF-8 are reported and left unchanged.
- A folder with code (other than TypeScript, JavaScript, or Kotlin) is refused; move its files with their own backend.

**TypeScript symbol rename**
- Only usages in projects the engine loads are renamed: keep every caller in the package's tsconfig and search for the old name afterwards.
- The tsconfig must be accepted by TypeScript 7. `baseUrl` and `moduleResolution: node10` are rejected with the engine's diagnostic. File moves are not affected.
- A name that clashes with or shadows another symbol is refused, and an ambiguous name asks for `--line`.

**General**
- File moves have no dry-run mode: changes are applied to disk immediately. `rename` supports `--dry-run`.
- Ordinary file-move backends may overwrite a target path. Rust `move-module` rejects existing targets during preflight.
- The tool does not walk into `node_modules/`, `target/`, `.git/`, or similar build/vendor directories when scanning for references.

---

## Prerequisites

| Language | Required | Install |
|---|---|---|
| TypeScript / JS | `bun` | [bun.sh](https://bun.sh) |
| Python | `rope` importable from `.venv` or `python3` | `pip install rope` |
| Python (fallback) | `pyrefly` (only if Rope is absent) | `pip install pyrefly` |
| Python (`rename`) | `basedpyright` | `pip install basedpyright` |
| Rust | `rust-analyzer` for symbol rename and ordinary file renames; semantic module support is embedded | `rustup component add rust-analyzer` |
| Go | `gopls` | `go install golang.org/x/tools/gopls@latest` |
| Dart | Dart SDK | [dart.dev/get-dart](https://dart.dev/get-dart) |
| Kotlin / Android | JetBrains Kotlin language server (`REFAC_KOTLIN_SERVER`), JDK 17+, a Gradle project; `ANDROID_HOME` for Android | [Kotlin server setup](Project_Manag/Docs/Setup/kotlin_Server.md) |
| Markdown | none | — |

`refac doctor <language>` prints these steps for the machine it runs on and proves the server starts. No specific minimum version is enforced for external language tools, but use recent releases. This checkout pins rust-analyzer 1.99.0 for ordinary Rust file renames and locks the embedded rust-analyzer crates in Cargo.

---

## How it works

The approach depends on the language:

**LSP-backed file moves (Rust, Go, Dart):** The tool starts a language server process, issues a rename request (`textDocument/rename` or `workspace/willRenameFiles`), applies the workspace edit the server returns, then moves the file on the filesystem. For batch operations, multiple renames are sent within a single server session with `textDocument/didChange` notifications between them to keep the server's view current. The tool never waits a fixed time: it sends the project's documents and waits for the server's own readiness signal (Dart `$/analyzerStatus`, gopls `$/progress` end, rust-analyzer `experimental/serverStatus`, pyrefly diagnostics). A server that never signals is an error after `REFAC_LSP_TIMEOUT_SECS` (default 300). For Dart the plan is checked before writing: if it would leave an import pointing at a missing file, nothing is changed.

**Semantic Rust modules:** `move-module` loads the Cargo workspace through embedded rust-analyzer crates, resolves the logical module and references through HIR, plans conventional module-tree edits and physical moves, then validates a fresh semantic load and the full Cargo workspace. Unsupported or ambiguous structures fail with a descriptive error rather than falling back to text-only guesses.

**Shared rename engine (Go, Rust, Python, Dart, Kotlin symbol rename):** `rename` (or `rename --batch`, which does this for each of its renames in one server session) starts the language server for the file's language, asks it for the references and the rename edits, shows it the renamed text in memory and asks for the references again, and writes only if the sets match (and every listed reference was edited). The server is shut down before anything is written. Python overrides are found with `textDocument/implementation` and renamed together; gopls is asked again when it answers under load with only part of the edits; the Dart server is made to finish analysing the changed documents first. See [Engine](Project_Manag/Docs/Features/Symbol_Rename/engine.md).

**TypeScript 7 native server (TypeScript / JavaScript symbol rename):** `rename` starts `tsc --lsp --stdio` from the locked `typescript-native` dependency, asks it for the rename edits, applies them in memory only, and asks for the references of the renamed declaration to prove that no clash or shadowing changed any meaning. Files are written only after that check, with rollback on a failed write. See [Symbol rename](Project_Manag/Docs/Features/TypeScript/symbol_Rename.md).

**Oxc (TypeScript / JavaScript):** A Bun helper parses configured sources with Oxc and resolves code imports with TypeScript, without a compiler Program or type checker. It plans the complete batch, edits module literals precisely, and verifies resolution after movement. Ordinary apply/verification failures roll back. Assets use Oxc Resolver; the supervisor enforces time and memory limits. See [TypeScript details](Project_Manag/Docs/Features/TypeScript/linker_TypeScript.md).

**Rope (Python):** The Rope refactoring library is invoked directly via Python. It performs the move and updates all import statements it can trace.

**Native (Markdown):** A CommonMark parser (`pulldown-cmark`) finds every link, image, reference definition, and the `href`/`src`/`srcset` values in raw HTML. The tool plans the full change (moves, then rewritten destinations written the way the author wrote them), applies it, and rolls back everything if a step fails. The same engine runs after any other backend has moved files, so a README that points at `src/app.py` follows the move. No external tooling required. See [Markdown](Project_Manag/Docs/Features/Markdown/linker_Markdown.md).

---

## Running tests

```bash
cargo test
bun install --cwd scripts --frozen-lockfile
bun run --cwd scripts test
bun run --cwd scripts typecheck
```

The suite covers unit tests and integration tests for all supported languages. Integration tests copy fixture projects into temp directories and run assertions on the resulting files. They live in six groups, one test program each: `cargo test --test cli`, `moves`, `rename`, `typescript`, `markdown`, `kotlin` (a part of a group with a filter, `cargo test --test rename go::`). A plain `cargo test` needs only the Rust toolchain (plus Bun for the TypeScript engine) and never skips silently: every test that needs a language server (gopls, the Dart SDK, rope, basedpyright, the Kotlin server) is `#[ignore = "needs X (run `refac doctor Y`)"]`, so it shows up as ignored in the summary. With the tools installed run them with `cargo test -- --ignored` (or `--include-ignored` for everything); without the tool they fail with an explanation of what was looked for.

**Disk and memory.** The repository is 3 MB; a fresh `cargo test` build writes 2.1 GB (`target/`) and peaks at 3.2 GB of RAM, because the Cargo profile drops debug information and the integration tests are grouped. Do not add a `.rs` file directly in `tests/` (it becomes another 90 MB program); see [Resource use](Project_Manag/Docs/Setup/resource_Use.md).

---

## Contributing

Issues and pull requests are welcome. There is no formal contribution guide yet — open an issue first if you are planning a significant change.

---

## License

Hippocratic License HL3 — see [LICENSE](LICENSE).
