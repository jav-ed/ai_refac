# Local Backend Workspace Decision

This is background context, not part of the normal setup path. Read this only when debugging local-backend workspace behavior or when changing the CI/CD design. For regular project integration, use [First-time setup](../first_Setup.md) and [Configure variables](../configure_Variables.md).

## Problem

We originally expected this shape to work:

```yaml
skip_clone: true

commands:
  - Code/CI_CD/Steps/01_Install.sh
```

The assumption was that `skip_clone: true` would make Woodpecker use the existing checkout as the working directory. That was wrong.

## What Actually Happens

`skip_clone: true` only disables Woodpecker's automatic clone step. The local backend still creates a temporary workflow workspace such as:

```text
/tmp/woodpecker-local-865567789/workspace
```

Then command steps start from that temporary workspace. Because that folder is not the repo checkout, plain relative script paths such as `Code/CI_CD/Steps/01_Install.sh` cannot resolve.

Woodpecker also sets `CI_WORKSPACE` to the temporary workspace and blocks overwriting that variable from step config, so `CI_WORKSPACE` is not a viable way to point at the real checkout.

## Trial: Git Root Detection

The older Jav_Web pipeline calls repo-owned scripts through absolute paths. Those scripts use their own location and then run:

```bash
git rev-parse --show-toplevel
```

That works only after execution has already entered a file path inside the real repo. It does not work from Woodpecker's temporary workspace because there is no `.git` parent above that temp folder.

## Trial: Derive Repo Path Automatically

Woodpecker provides repository metadata such as clone URLs and commit SHA. In theory, a wrapper could search approved filesystem roots for a checkout whose Git remote matches the pipeline repo.

We rejected that for the current template because it adds a resolver, search roots, ambiguity handling, and commit synchronization checks. It is more machinery than the current single-machine local-backend deployment needs.

## Decision

Use one explicit variable in `.woodpecker.yml`:

```yaml
CI_REPO_PATH: /absolute/path/to/project/repo
```

Every step calls the matching script through that absolute checkout path:

```yaml
commands:
  - "$CI_REPO_PATH/Code/CI_CD/Steps/01_Install.sh"
```

After the script starts, `Code/CI_CD/Lib/runtime.sh` derives the project root from the script location and loads `Code/CI_CD/Config/env.env`.

This keeps the pipeline simple:

- `.woodpecker.yml` owns the one value needed before repo scripts can run.
- `Code/CI_CD/Config/env.env` owns project/deploy values after script runtime loads.
- the template fails clearly until `CI_REPO_PATH` and `CI_DEPLOY_DOMAIN` are deliberately completed.

## Revisit Only If

Reconsider this design if:

- the local backend starts running many projects across many different checkout roots,
- the checkout path changes often,
- multiple Woodpecker agents are introduced,
- clone-based or container-based CI replaces the local backend,
- the team accepts a maintained repo-path resolver.
