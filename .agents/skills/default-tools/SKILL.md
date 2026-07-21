---
name: default-tools
description: Hub for tools used across most tasks in this repo, currently covering tool/version management (mise), git hooks (hk), secrets (sops as default, plus fnox and raw age for specific cases), and web automation (playwright-cli). Use when starting any task that touches `mise.toml`, `hk.pkl`, credentials, browsers, screenshots, or when you want a quick map of the recurring tooling before deciding which skill to load.
---

# default-tools

Some tools come up in nearly every task in this repo. This skill is the map: read it once at the start of a task, see which tool the work calls for, then either follow the inline guidance or jump to the deeper reference linked at the end of each section.

## Mise — runtime and tool version management

Read this section when:

- A project has a `mise.toml` and you need to install or run its tools.
- You need to install, upgrade, or pin a tool version (Node, Bun, Go, Python, etc.).
- You hit `command not found` or `untrusted config` errors related to mise.
- You're setting up a new machine or server where mise is not yet present.

Mise replaces language-specific version managers (nvm, pyenv, rustup) with one tool. In this repo it runs in two modes: standard install on local dev machines (shell hooks active), and shared system-wide install at `/opt/mise` on the server (no shell hooks; static PATH in `/etc/environment`).

→ [Mise](Mise/linker_Mise.md): core commands, local vs server differences, gotchas (especially env-var visibility under shims), troubleshooting, and the server installation playbook.

## Hk — git hooks (lint, format, validate on commit/push)

Read this section when:

- A project has `hk.pkl` and you need to understand or modify what runs on commit/push.
- You're setting up hooks on a fresh machine and need `hk install --global`.
- You hit hooks firing twice, hooks not running at all, or linters not finding their tools.
- You need to bypass a hook for a single commit (`HK=0 git commit`).

hk is a parallel git hook runner by jdx, configured in Pkl. It uses file-level read/write locks so multiple linters can run safely in parallel — unlike pre-commit (sequential) or lefthook (unsafe parallel). Installed via mise; the global install (`hk install --global`) requires Git 2.54+ and is the recommended setup since `hk.pkl`-less repos are silent no-ops.

→ [Hk](Hk/linker_Hk.md): core commands, `hk.pkl` essentials, install modes (global vs per-repo), bypass mechanisms, mise integration (`HK_MISE=1`), and troubleshooting.

## Secrets — credentials, API tokens, encrypted config

Read this section when:

- You need to read or set a secret value (API key, token, password).
- You need to run a process with secrets injected as env vars.
- You're editing encrypted config files and need to decrypt or re-encrypt.
- You see `.sops.yaml`, `.age` files, `fnox.toml`, or references to `sops edit` / `fnox exec` / `age -d` in the codebase.

### Three tools, one structural distinction

All three use age underneath. They differ on **how the secret reaches the application**:

- **sops (default for new work)** — encrypts individual values *inside* a config file the app already reads. After `sops decrypt` (or via library bindings) `cfg.SmtpPass` is a normal string field on the parsed struct, no extra step in the code. CNCF graduated, ~10 years old, multi-maintainer. *Caveat: standard sops does not natively support TOML — only YAML, JSON, INI, ENV, and BINARY.*
- **raw age** — encrypts a whole file end-to-end. The app reads it normally after a one-shot decryption. Used in this repo for Dynaconf `.toml` files because sops can't field-encrypt TOML.
- **fnox** — injects secrets as environment variables. `fnox exec -- ./binary` decrypts everything and exposes values via `os.Getenv("KEY")`. Used for daemons, CLIs, and scripts that want env-var-shaped input. Backed by age plus 19 other providers (Vaultwarden, KMS, etc.).

The structural choice: *secrets-via-file* (sops, raw age) puts the value inside the file the code reads. *Secrets-via-environment* (fnox) injects them into the process before the binary starts. Pick based on what the consuming code wants, not on aesthetics.

### Decision table

| | **sops** (default) | **raw age** | **fnox** |
|---|---|---|---|
| How code consumes the secret | Reads it as a normal field in the parsed config | Reads it as a normal field (after whole-file decrypt) | Reads `os.Getenv("KEY")` |
| Supported file formats | YAML, JSON, INI, ENV | Any (treated as opaque blob) | n/a — values live in `fnox.toml` |
| Granularity | Per-field within a file | Whole-file | Per-key |
| Edit flow | `sops edit file.yaml` opens decrypted in `$EDITOR`, re-encrypts on save | Decrypt → edit plaintext → re-encrypt | `fnox edit KEY` |
| Best fit | Structured app config or per-tenant config with a few secret fields | TOML / Dynaconf / one-off whole-file encryption | Daemons, CLIs, scripts wanting env vars |

### Where each is used

**Default for new work:** sops on YAML / JSON / INI / ENV, with recipients declared once in a root `.sops.yaml`.
→ [Sops workflow](Secrets/Sops/workflow.md): age key setup, `.sops.yaml` creation rules, edit flow, key rotation, TOML caveat, known recipients.

**This repo's existing Dynaconf `.toml` secrets:** raw age via `Input/Dynaconf/manage_secrets.sh`. Stays as-is — sops doesn't support TOML, and the whole-file-decrypt-at-startup pattern works fine for Dynaconf.
→ [Age workflow](Secrets/Age/workflow.md): manage_secrets.sh commands, recipients, identity, adding a new device.

**`03_Post_Sched` and other env-var-driven services:** fnox.
→ [Fnox](Secrets/Fnox/linker_Fnox.md): setup, core commands, g12 mise bridge, troubleshooting.

## Web automation — browser, screenshots, form filling, scraping

Read this section when:

- You need to navigate a web page, click, fill forms, or extract content.
- You need a screenshot of a webpage or a running dev server.
- You're doing visual testing for a webpage project.

### The playwright-cli skill stays external

Unlike Secrets above, the `playwright-cli` skill is **not** copied into this hub. It is a standalone external skill, kept that way so it stays in sync with upstream changes to the tool.

**Skill check + self-install.** Do not assume `playwright-cli` is installed just because a previous session used it. Check first; if the binary is missing or the skill is not loaded, run from the project root:

```bash
playwright-cli install --skills
```

That is the canonical install step. Once it succeeds, load the `playwright-cli` skill for the full command reference (open, snapshot, click, fill, screenshot, network mocking, tabs, tracing, etc.).

### Scratch/ — where visual outputs go

All screenshots, mockups, and audits go under `Scratch/` at the repo root. The whole folder is **gitignored end-to-end**; nothing inside is ever committed. If a file there matters, copy it to a tracked location before relying on it.

Three subfolders, set up by `doc-start`'s bootstrap:

| Subfolder | Use for |
|---|---|
| `Scratch/Screenshots/` | Playwright (or other) screenshots of the running app |
| `Scratch/Design/` | Design references and mockups pulled in for visual comparison |
| `Scratch/Audit/` | Audit reports and snapshots (accessibility, performance, etc.) |

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
