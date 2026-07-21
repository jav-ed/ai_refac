# TypeScript / JavaScript

The TypeScript backend delegates to `ts-morph` via a Bun script (`scripts/ts_refactor.ts`). It handles both file and directory moves and rewrites import paths across the project.

## Required tooling

- `bun` — checked at `bun` in PATH, then `~/.bun/bin/bun`
- `ts-morph` 28 — auto-installed via `bun install` on first run if not found in `scripts/node_modules`
- `tsconfig.json` — optional but strongly recommended (see below)

## How project loading works

| Condition | What happens |
| :--- | :--- |
| `tsconfig.json` present, ≤2,000 source files | Full project loaded via tsconfig — all inbound references updated |
| `tsconfig.json` present, >2,000 source files, **file move** | Only the moved file is loaded — cross-project reference updates are **skipped** |
| `tsconfig.json` present, **directory move** | Full project always loaded regardless of size |
| No `tsconfig.json` | Falls back to globbing all `*.ts/.tsx/.js/.jsx` under `--project-path` |

The 2,000-file threshold exists because loading a very large tsconfig project causes multi-minute freezes with no output. It counts the source files selected by the package's `tsconfig.json`, so ignored nested repositories and tooling outside that config do not inflate the project size.

## Performance status

Version 28 does not claim a move-performance improvement; it primarily upgrades the embedded compiler to TypeScript 6. Upstream still tracks slow `sourceFile.move` and directory-move behavior in [issue 1613](https://github.com/dsherret/ts-morph/issues/1613) and [issue 953](https://github.com/dsherret/ts-morph/issues/953), so Refac keeps the 30-file hard limit.

A local comparison on the Shadi Intake project found no consistent advantage from the Node `tsx` runner. `tsx` was about 10% faster for one 27-file directory move, but Bun was slightly faster when the same 27 files were moved individually in one batch. Both produced identical trees, so Refac continues to use Bun rather than adding a second runtime dependency.

## Key limits

- **Large file-move projects**: cross-project reference updates are skipped when >2,000 files. Only the moved file's own import paths are rewritten. Pass `--project-path` to a sub-package root (the folder with `tsconfig.json`) rather than the monorepo root to stay under the threshold.
- **Timeout**: 5-minute hard limit. If it fires, the same advice applies — narrow `--project-path`.
- **No tsconfig**: without a tsconfig, compiler options default to `allowJs: true`. Type-aware reference resolution is weaker.
- **Batch size**: at most 30 contained TypeScript/JavaScript source files. Directory contents are counted recursively before mutation, and the measured count is included in successful output.
- **Aliases**: imports using aliases declared in `compilerOptions.paths` are rewritten explicitly. A stale moved alias import makes the operation fail instead of reporting success.
