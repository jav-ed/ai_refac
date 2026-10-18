# First-Time Setup

End-to-end checklist for bringing a new Astro repo into the existing Woodpecker pipeline. Use this file for the normal static-site setup path. For a compiled Rust utility, stop here and use [Rust binary tool](rust_Binary_Tool.md). For the Astro variable map, use [Configure variables](configure_Variables.md). For background on why the local backend needs `CI_REPO_PATH`, use [Local backend workspace decision](Detail/local_Backend_Workspace_Decision.md) only when debugging or changing the design.

Assumes:

- the Astro repo already exists on the forge,
- Woodpecker server and agent already run on the agent machine,
- Woodpecker can access the forge,
- the deploy host is reachable over SSH,
- the runtime static-site root already follows the `Web_Layer/20_Static_Websites/` layout.

Do not repair Forgejo/Woodpecker platform setup from this file unless the user asks or an error clearly contradicts the assumption. In that case, stop here and route through [Platform](../../Platform/linker_Platform.md).

If this is the repo's first CI/CD setup, the Woodpecker activation or Forgejo webhook may not exist yet. That is normal. Copy and validate the local files first, then use [trigger checks](../Trigger/repo_Trigger_Checks.md) to confirm or repair the trigger path before expecting a push to run.

## Contents

- [Copy the template](#1-copy-the-template)
- [Configure required variables](#2-configure-required-variables)
- [Confirm local shortcuts](#3-confirm-local-shortcuts)
- [Confirm shared tools](#4-confirm-shared-tools)
- [Dry-run locally](#5-dry-run-the-pipeline-locally)
- [Confirm the runtime folder](#6-confirm-runtime-site-folder)
- [Confirm the trigger path](#7-confirm-trigger-path)
- [First production push](#8-first-production-push)

## 1. Copy The Template

From the new Astro repo root:

```bash
SKILL_TEMPLATE="<path-to-ci-cd-skill>/Template"

mkdir -p Code
cp "$SKILL_TEMPLATE/.woodpecker.yml" .
cp "$SKILL_TEMPLATE/justfile" .
cp -r "$SKILL_TEMPLATE/Code/CI_CD" Code/
chmod +x Code/CI_CD/Steps/*.sh
```

The template includes `justfile` as the mandatory local shortcut layer for `just Step_02_Build` style commands. Woodpecker still calls `Code/CI_CD/Steps/*.sh` directly.

Expected shape:

```text
<repo-root>/
|-- .woodpecker.yml
|-- justfile
`-- Code/
    `-- CI_CD/
        |-- Config/
        |   `-- env.env
        |-- Lib/
        `-- Steps/
```

## 2. Configure Required Variables

Open [Configure variables](configure_Variables.md) and complete the mandatory project values.

At minimum:

- `.woodpecker.yml`: set `CI_REPO_PATH`.
- `Code/CI_CD/Config/env.env`: set `CI_DEPLOY_DOMAIN` and confirm the deploy/shared-tool values.

Do not move `CI_REPO_PATH` into the script env file. Woodpecker needs it before it can start the repo-owned scripts by absolute path.

## 3. Confirm Local Shortcuts

From the repo root:

```bash
just --list
```

Expected: recipes such as `Step_01_Install`, `Step_02_Build`, `Step_03_Og_Images`, `Step_04_Compress`, and `Step_05_Deploy` are listed.

Just is only a local alias layer here. It should not load CI env values or install CI tools.

## 4. Confirm Shared Tools

The compressor and OG generator are org-wide tools, not project dependencies:

```bash
source Code/CI_CD/Config/env.env
test -x "$CI_COMPRESSOR_BIN"
test -f "$CI_OG_IMAGE_DIR/src/main.js"
```

If either check fails, build or install the tool as the `web-comp-tools` skill says (its `path_File.md` names the folder and binary these two variables must point to) before continuing.

## 5. Dry-Run The Pipeline Locally

From the repo root:

```bash
just Step_01_Install
just Step_02_Build
just Step_03_Og_Images
just Step_04_Compress
```

Calling `Code/CI_CD/Steps/*.sh` directly is also acceptable for debugging. The scripts are the source of truth and are what Woodpecker runs.

Do not run `just Step_05_Deploy` until the project is approved for production deployment. The deploy step prints the intended site, lane, release path, and active `dist` symlink target before it writes.

## 6. Confirm Runtime Site Folder

The deploy target is derived as:

```text
CI_DEPLOYMENT_ROOT / CI_DEPLOY_DOMAIN / main
```

The runtime folder should contain the domain folder with its `domain.caddy` and the mandatory `main` lane. The deploy script creates `releases/` and the `dist` symlink itself (see [Release layout, retention and rollback](../../Architecture/release_Layout.md)), but the domain folder and Caddy route should be intentional runtime setup, not guessed by the pipeline.

If the site emits same-origin proxy paths for private services, the matching `domain.caddy` routes are part of runtime setup. Example for self-hosted Umami behind private infrastructure:

```caddy
handle /assets/js/i18n.js {
    rewrite * /i18n.js
    reverse_proxy 127.0.0.1:8089
}

handle /api/locale {
    reverse_proxy 127.0.0.1:8089
}
```

The Astro source should emit `/assets/js/i18n.js` with `data-host-url="/"`; it should not emit the private analytics hostname. This keeps public browsers on the public site origin while Caddy owns the private hop.

## 7. Confirm Trigger Path

Use [repo trigger checks](../Trigger/repo_Trigger_Checks.md) to confirm:

- `.woodpecker.yml`, `justfile`, and `Code/CI_CD/` are committed,
- the repo is enabled in Woodpecker,
- the Forgejo webhook exists and is active.

If either Woodpecker activation or Forgejo webhook is missing, use [repo trigger setup](../Trigger/repo_Trigger_Setup.md).

## 8. First Production Push

Commit and push when the project is ready for the guarded `main` trigger check:

```bash
git checkout main
git add .woodpecker.yml justfile Code/CI_CD
git commit -m "add CI/CD pipeline"
git push origin main
```

After the run completes:

```bash
ssh <deploy-host> "readlink <deployment-root>/<domain>/main/dist"
ssh <deploy-host> "curl -sI https://<domain> | head -5"
```

For client work, use this only once the deploy target and public domain are approved. Safe non-production inspection targets are deferred design work and must use a Zetun-owned hostname plus NetBird-only access.
