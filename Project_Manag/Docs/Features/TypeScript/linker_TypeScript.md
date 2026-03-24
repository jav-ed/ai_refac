# TypeScript / JavaScript

The TypeScript backend delegates to `ts-morph` via a Bun script (`scripts/ts_refactor.ts`). It handles both file and directory moves and rewrites import paths across the project.

## Required tooling

- `bun` — checked at `bun` in PATH, then `~/.bun/bin/bun`
- `ts-morph` — auto-installed via `bun install` on first run if not found in `scripts/node_modules`
- `tsconfig.json` — optional but strongly recommended (see below)

## How project loading works

| Condition | What happens |
| :--- | :--- |
| `tsconfig.json` present, ≤500 source files | Full project loaded via tsconfig — all inbound references updated |
| `tsconfig.json` present, >500 source files, **file move** | Only the moved file is loaded — cross-project reference updates are **skipped** |
| `tsconfig.json` present, **directory move** | Full project always loaded regardless of size |
| No `tsconfig.json` | Falls back to globbing all `*.ts/.tsx/.js/.jsx` under `--project-path` |

The 500-file threshold exists because loading a large tsconfig project causes multi-minute freezes with no output. The threshold counts files under `--project-path`, excluding `node_modules`, `dist`, `build`, `.next`, and `.git`.

## Key limits

- **Large file-move projects**: cross-project reference updates are skipped when >500 files. Only the moved file's own import paths are rewritten. Pass `--project-path` to a sub-package root (the folder with `tsconfig.json`) rather than the monorepo root to stay under the threshold.
- **Timeout**: 5-minute hard limit. If it fires, the same advice applies — narrow `--project-path`.
- **No tsconfig**: without a tsconfig, compiler options default to `allowJs: true`. Type-aware reference resolution is weaker.
