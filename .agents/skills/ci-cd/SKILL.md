---
name: ci-cd
description: Defines the Woodpecker CI/CD workflow for Astro static sites and compiled Rust utilities deployed from a local runner. Use when adding a project pipeline, checking Forgejo/Woodpecker triggers, debugging deploy or binary-install flow, or working with Zetun CI/CD platform access.
---

# ci-cd

This skill covers the project-pipeline workflow for Zetun Astro repos and compiled Rust utility repos. It covers Woodpecker local-backend pipelines, Forgejo-triggered runs, project-level `Code/CI_CD/` scripts, mandatory `justfile` shortcuts, static deploys to Caddy, atomic binary installation, and the platform access needed to inspect or repair triggers.

Default assumption: Forgejo and Woodpecker already exist, the user has access, and the platform is healthy. Do not start by repairing platform services. Enter [Platform](Platform/linker_Platform.md) only when the user asks for shared setup/repair or an error clearly shows Forgejo/Woodpecker itself is broken.

Current active deployment target: `main` to production. The optional `test` branch / internal non-production environment is WIP design only; do not configure it unless the user explicitly asks to work on that deferred design.

## Project Shape

Every integrated repo should have:

```text
.woodpecker.yml      # Woodpecker entrypoint; calls scripts through CI_REPO_PATH
justfile             # mandatory local shortcut recipes
Code/CI_CD/          # implementation: env, tasks, shell steps, shared libs
```

Repos with several local commands may keep the root `justfile` as an import index and place focused recipes under `Code/Just/`. Just remains a human-facing wrapper; Woodpecker always calls `Code/CI_CD/Steps/*.sh` directly.

The shell scripts are the CI/CD execution layer. Just is only a local shortcut wrapper for humans who prefer short commands such as `just Step_02_Build`; it should not own project env loading or tool installation for CI.

## Common Routes

- Already integrated and only need normal operation: use [Active use](Application/Active_Use/daily_Workflow.md).
- Need to know whether a push produced a pipeline, which step failed, or what a step's log says: use [Read a pipeline run's output](Application/Active_Use/read_Pipeline_Output.md), which reads Woodpecker through a GET-only helper.
- Adding CI/CD to a repo without `Code/CI_CD/`: use [Integration](Application/Integration/first_Setup.md), which copies the template, completes `.woodpecker.yml` and `Code/CI_CD/Config/env.env`, runs local script checks, and then routes to trigger checks. Use [Configure variables](Application/Integration/configure_Variables.md) for the variable map.
- Adding CI/CD to a compiled Rust utility: use [Rust binary tool](Application/Integration/rust_Binary_Tool.md), which pins the Rust environment explicitly, separates build from atomic installation, provides modular Just recipes, and proves the non-interactive HOME boundary before the first push.
- Adding or operating semantic IndexNow notifications: use [IndexNow notifications](Application/Integration/indexNow_Notifications.md) for the normal path. Follow its `Detail/` route only when changing comparison, bootstrap, deployment, locking, or report invariants.
- Repo has files but Woodpecker does not run: use [Trigger checks](Application/Trigger/repo_Trigger_Checks.md), then [Trigger setup](Application/Trigger/repo_Trigger_Setup.md) only after a missing webhook or Woodpecker activation is confirmed.
- Forgejo, Woodpecker, access, or service health looks broken: use [Platform](Platform/linker_Platform.md) and stop treating it as an application problem.
- Copyable baselines live in [Template](Template/): the root is the Astro static-site template, [Template/IndexNow](Template/IndexNow/) is its opt-in notification overlay, and [Template/Rust_Binary](Template/Rust_Binary/) is the compiled-tool template. Templates intentionally contain broken placeholders where project values must be reviewed.

## Main Path

For a new or repaired Astro project pipeline:

1. Confirm platform assumptions only if needed: Forgejo/Woodpecker running, user access available.
2. If `Code/CI_CD/` is missing, copy the template through [Integration](Application/Integration/first_Setup.md).
3. Complete variables through [Configure variables](Application/Integration/configure_Variables.md): `.woodpecker.yml` owns `CI_REPO_PATH`; `Code/CI_CD/Config/env.env` owns project/deploy values.
4. Run local checks without pushing: `just Step_01_Install`, `just Step_02_Build`, `just Step_03_Og_Images`, and `just Step_04_Compress`, or call the matching `Code/CI_CD/Steps/*.sh` scripts directly.
5. Confirm `.woodpecker.yml`, `justfile`, and `Code/CI_CD/` are committed on `main`.
6. Confirm or create the trigger path: Woodpecker repo activation plus Forgejo webhook.
7. Trigger the required Woodpecker run on `main`; deploy is step `05_Deploy`.

When secret-backed commands are needed, read the default-tools Fnox docs first, then use the skill-owned `Platform/Secrets/fnox.toml` documented in [Secrets and access](Platform/Secrets/secrets_And_Access.md). This file is intentionally inside the skill so the skill folder remains portable.

## Navigation

- [Platform](Platform/linker_Platform.md): shared Forgejo/Woodpecker service checks, API discovery, and secret-backed access.
- [Application](Application/linker_Application.md): one Astro repo's integration, trigger checks/setup, deploy, rollback, verification, and debugging routes.
- [Optional internal test environment](Deferred/test_Environment.md): WIP notes for a possible `test` branch deployment target, including the distinction between private/owned domains and client projects; not active setup instructions.
- [Architecture](Architecture/architecture.md): Woodpecker + just shape, local backend assumptions, `skip_clone: true`, release naming, atomic symlink swap, and known tradeoffs.
- [Limitations](Deferred/limitations.md): current single-machine/single-worker scope and what must change before multi-machine, multi-worker, or containerized CI use.
- [Template](Template/): copyable `.woodpecker.yml`, `justfile`, and `Code/CI_CD/` folder for Astro project pipelines.
