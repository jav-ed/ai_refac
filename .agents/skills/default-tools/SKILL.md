---
name: default-tools
description: "Hub for recurring repository tooling: shared system-tool installation through mise and Just, git hooks with hk, and web automation with Playwright CLI. Use when installing or updating the shared tool baseline, editing the shared mise manifest or hk.pkl, writing Just recipes, or working with browsers and screenshots."
---

# default-tools

Some tools come up in nearly every task in this repo. This skill is the map: read it once at the start of a task, see which tool the work calls for, then either follow the inline guidance or jump to the deeper reference linked at the end of each section. Credentials and encrypted config are not handled here: the `secrets` skill owns sops, age and fnox.

## Mise: system tool installation

Read this section when:

- You need to install or update a shared CLI or development tool.
- You encounter a mise shim or project-level `mise.toml` reference.
- You are migrating a command to a stable system path.

Mise is an acquisition and update manager only. System tools live under `/usr/local/share/mise/installs` and are exposed through explicit `/usr/local/bin` links. Do not use activation, shims, project version lookup, environment injection, or mise tasks. Bun and Oh My Zsh use their official installers; Just owns repository tasks.

The shared tool list is the `[tools]` table of `/etc/mise/config.toml`, an installed copy of a manifest. Add a tool there with a purpose comment (or in the tracked manifest, when a repository keeps one), install it as a system tool, then link its approved executable. Installing and updating need root: run the documented commands with sudo, or hand them to the user when sudo is unavailable.

→ [Mise](Mise/linker_Mise.md): install method order, layout, the exact install, link, update and verify commands, and boundaries.

## Just: recurring commands

Read this section when:

- You are about to run a long command or command sequence a second time.
- You need to add, change or find a Just recipe.

- Look before adding: `just --list`, `just --groups`, `just --show <recipe>`. Reuse or extend the recipe that owns the job. Exploratory commands stay in `Scratch/` until they recur.
- Group recipes by purpose in `Code/Just/*.just` files imported by the root `justfile`, with matching `[group('purpose')]` attributes so `just --list` shows the structure. Reuse a cohesive file instead of making one file per recipe.
- Keep recurring long paths in descriptive variables in the owning file, derived from `justfile_directory()`. Task-specific Scratch paths are parameters, not permanent defaults.
- Comment above each recipe's attributes: purpose, meaningful inputs, side effects and prerequisites. Comment non-obvious ordering and workarounds where they occur.
- Keep recipes short. Put long lifecycle logic in a focused script, quote path arguments, propagate failures, and report missing inputs descriptively. When you introduce a workflow, run it and verify the result.

## Hk — git hooks (lint, format, validate on commit/push)

Read this section when:

- A project has `hk.pkl` and you need to understand or modify what runs on commit/push.
- You're setting up hooks on a fresh machine and need `hk install --global`.
- You hit hooks firing twice, hooks not running at all, or linters not finding their tools.
- You need to bypass a hook for a single commit (`HK=0 git commit`).

hk is a parallel git hook runner by jdx, configured in Pkl. Install its binary through the system tool policy and expose it at `/usr/local/bin/hk`. The global install (`hk install --global`) requires Git 2.54+.

→ [Hk](Hk/linker_Hk.md): core commands, `hk.pkl` essentials, install modes, bypass mechanisms, and troubleshooting.

## Web automation — browser, screenshots, form filling, scraping

Read this section when:

- You need to navigate a web page, click, fill forms, or extract content.
- You need a screenshot of a webpage or a running dev server.
- You're doing visual testing for a webpage project.

### The playwright-cli skill stays external

The `playwright-cli` skill is **not** copied into this hub. It is a standalone external skill, kept that way so it stays in sync with upstream changes to the tool.

The Playwright CLI binary is installed as a shared system tool (`npm:@playwright/cli` in the mise manifest). After upgrading the binary, refresh the repository's standalone agent skill when its version warning asks you to:

```bash
playwright-cli install --skills=agents
```

Then load the `playwright-cli` skill for its full command reference (open, snapshot, click, fill, screenshot, network mocking, tabs, tracing, etc.).

### Scratch/ — where visual outputs go

All screenshots, mockups, and audits go under `Scratch/` at the repo root. The whole folder is **gitignored end-to-end**; nothing inside is ever committed. If a file there matters, copy it to a tracked location before relying on it. The `doc-start` skill owns the folder layout (`Screenshots/`, `Design/`, `Audit/`, `Agent_Tasks/`) and the manual `just scratch-clean` reset: read the `doc-start` skill for the cleanup contract.

Always route screenshot output to `Scratch/Screenshots/`, never to a tracked path:

```bash
playwright-cli screenshot --filename=Scratch/Screenshots/<page>.png
```

### Per-project specifics

For webpage projects, the project-specific dev server command, port, and any extra setup live in `Project_Manag/Docs/Setup/visual_Testing.md` (created on demand by `doc-start`). Read that file before running visual tests in an unfamiliar project — port numbers and dev-server commands vary by stack.

Generic dev-server check:

```bash
ss -tlnp | grep <port>
```

Any output means the server is up; empty output means start it (`bun run dev`, `npm run dev`, etc.).
