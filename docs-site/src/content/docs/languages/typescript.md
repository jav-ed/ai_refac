---
title: TypeScript / JavaScript
description: How refac handles TypeScript and JavaScript file moves using ts-morph.
---

The TypeScript backend delegates to [ts-morph](https://ts-morph.com/) via a Bun script. It handles both file and directory moves and rewrites import paths across the project using the TypeScript Compiler API.

## Required tooling

| Tool | Notes |
|---|---|
| `bun` | Checked at `bun` in PATH, then `~/.bun/bin/bun` |
| `ts-morph` | Auto-installed via `bun install` on first run |
| `tsconfig.json` | Optional but strongly recommended — see below |

## How project loading works

| Condition | What happens |
|---|---|
| `tsconfig.json` present, ≤ 500 source files | Full project loaded — all inbound references updated |
| `tsconfig.json` present, > 500 source files, **file move** | Only the moved file is loaded — cross-project reference updates are **skipped** |
| `tsconfig.json` present, **directory move** | Full project always loaded regardless of size |
| No `tsconfig.json` | Falls back to globbing all `*.ts/.tsx/.js/.jsx` under `--project-path` |

The 500-file threshold exists because loading a large tsconfig project causes multi-minute freezes with no output. The count excludes `node_modules`, `dist`, `build`, `.next`, and `.git`.

## Limitations

**Large projects:** When a file move involves more than 500 source files, cross-project reference updates are skipped. Only the moved file's own import paths are rewritten. Files that import the moved file are not updated.

**Workaround:** Point `--project-path` at the sub-package that owns `tsconfig.json` rather than the monorepo root. This keeps the file count under the threshold and enables full reference resolution.

**Timeout:** There is a 5-minute hard limit on the Bun script. If it fires, the same advice applies — narrow `--project-path`.

**No tsconfig:** Without a tsconfig, compiler options default to `allowJs: true`. Type-aware reference resolution is weaker in this mode.
