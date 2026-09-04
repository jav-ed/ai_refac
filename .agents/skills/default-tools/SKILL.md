---
name: default-tools
description: "Hub for recurring repository tooling: shared system-tool installation through mise and Just, git hooks with hk, secrets with Sops, fnox, or age, and web automation with Playwright CLI. Use when installing or updating the shared tool baseline, editing system_Tools.toml or hk.pkl, handling credentials, or working with browsers and screenshots."
---

# default-tools

Some tools come up in nearly every task in this repo. This skill is the map: read it once at the start of a task, see which tool the work calls for, then either follow the inline guidance or jump to the deeper reference linked at the end of each section.

## Mise: system tool installation

Read this section when:

- You need to install or update a shared CLI or development tool.
- You encounter a mise shim or project-level `mise.toml` reference.
- You are migrating a command to a stable system path.

Mise is an acquisition and update manager only. System tools live under `/usr/local/share/mise/installs` and are exposed through explicit `/usr/local/bin` links. Do not use activation, shims, project version lookup, environment injection, or mise tasks. Bun and Oh My Zsh use their official installers; Just owns repository tasks.

The tracked source of truth is `Project_Manag/Tools/Mise/system_Tools.toml`. Use the repository recipes rather than editing `/etc/mise/config.toml` directly:

```bash
just system-tools-install
just system-tools-update
```

→ [Mise](Mise/linker_Mise.md): system-install rules and the canonical operational reference.

## Hk — git hooks (lint, format, validate on commit/push)

Read this section when:

- A project has `hk.pkl` and you need to understand or modify what runs on commit/push.
- You're setting up hooks on a fresh machine and need `hk install --global`.
- You hit hooks firing twice, hooks not running at all, or linters not finding their tools.
- You need to bypass a hook for a single commit (`HK=0 git commit`).

hk is a parallel git hook runner by jdx, configured in Pkl. Install its binary through the system tool policy and expose it at `/usr/local/bin/hk`. The global install (`hk install --global`) requires Git 2.54+.

→ [Hk](Hk/linker_Hk.md): core commands, `hk.pkl` essentials, install modes, bypass mechanisms, and troubleshooting.

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

Fnox normally discovers `fnox.toml` by walking up the directory tree. When a repository keeps multiple secret sets or uses a descriptive filename, pass the file explicitly with `-c` or `--config` on every command. The filename does not need to be `fnox.toml`:

```bash
fnox get -c rishta_Worker_Mails.toml MARIA_BUTT_EMAIL_PASSWORD
fnox exec -c rishta_Worker_Mails.toml -- command
```

→ [Fnox](Secrets/Fnox/linker_Fnox.md): setup, core commands, explicit execution, and troubleshooting.

## Web automation — browser, screenshots, form filling, scraping

Read this section when:

- You need to navigate a web page, click, fill forms, or extract content.
- You need a screenshot of a webpage or a running dev server.
- You're doing visual testing for a webpage project.

### The playwright-cli skill stays external

Unlike Secrets above, the `playwright-cli` skill is **not** copied into this hub. It is a standalone external skill, kept that way so it stays in sync with upstream changes to the tool.

The Playwright CLI binary is part of `system_Tools.toml` and is installed with `just system-tools-install`. After upgrading the binary, refresh the repository's standalone agent skill when its version warning asks you to:

```bash
playwright-cli install --skills=agents
```

Then load the `playwright-cli` skill for its full command reference (open, snapshot, click, fill, screenshot, network mocking, tabs, tracing, etc.).

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
