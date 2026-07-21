# TypeScript / JavaScript

The TypeScript backend delegates to `ts-morph` via a Bun script (`scripts/ts_refactor.ts`). It handles both file and directory moves and rewrites import paths across the project.

## Required tooling

- `bun` — checked at `bun` in PATH, then `~/.bun/bin/bun`
- `ts-morph` 28 — auto-installed via `bun install` on first run if not found in `scripts/node_modules`
- `tsconfig.json` — required for complete local caller and alias coverage

## How project loading works

| Condition | What happens |
| :--- | :--- |
| `tsconfig.json` present | Every configured source file is loaded; recursive dependency discovery is skipped to reduce RAM |
| No `tsconfig.json` | Falls back to globbing all `*.ts/.tsx/.js/.jsx` under `--project-path` |

Point `--project-path` at the package that owns the authoritative tsconfig. Its `include` or `files` configuration must cover all local TS/JS files that participate in imports. External packages such as React or SolidJS remain package imports and do not need to be included.

## Performance status

Version 28 does not claim a move-performance improvement; it primarily upgrades the embedded compiler to TypeScript 6. Upstream still tracks slow `sourceFile.move` and directory-move behavior in [issue 1613](https://github.com/dsherret/ts-morph/issues/1613) and [issue 953](https://github.com/dsherret/ts-morph/issues/953), so Refac keeps the 30-file hard limit.

A local comparison on the Shadi Intake project found no consistent advantage from the Node `tsx` runner. `tsx` was about 10% faster for one 27-file directory move, but Bun was slightly faster when the same 27 files were moved individually in one batch. Both produced identical trees, so Refac continues to use Bun rather than adding a second runtime dependency.

## Key limits

- **Project size**: there is no partial-load success path. File and directory moves retain the complete tsconfig source set so external callers remain visible.
- **Timeout**: 5-minute hard limit. If it fires, narrow `--project-path` to the package that owns the relevant tsconfig.
- **No tsconfig**: without a tsconfig, compiler options default to `allowJs: true`. Type-aware reference resolution is weaker.
- **Batch size**: at most 30 contained TypeScript/JavaScript source files. Directory contents are counted recursively before mutation, and the measured count is included in successful output.
- **Aliases**: file and directory imports using aliases declared in `compilerOptions.paths` are rewritten explicitly. A stale moved alias import makes the operation fail instead of reporting success.
