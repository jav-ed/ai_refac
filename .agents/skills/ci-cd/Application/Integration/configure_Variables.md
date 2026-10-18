# Configure Variables

Use this during first-time integration after copying the template. This file is practical setup detail: it lists which variables must be changed, where they live, and which values must not be user-defined.

## Variable Ownership

Variables live in two places because they are needed at different phases.

| Location | Phase | Owns |
|---|---|---|
| `.woodpecker.yml` | Before repo scripts can start | `CI_REPO_PATH` only |
| `Code/CI_CD/Config/env.env` | After the repo script starts and runtime loads | deploy target, shared tool paths, retention |

Do not duplicate `CI_REPO_PATH` in `Code/CI_CD/Config/env.env`. The script env file cannot be loaded until Woodpecker has already used `CI_REPO_PATH` to start a repo-owned script by absolute path.

## `.woodpecker.yml`

Set:

```yaml
variables:
  - &base_step
    image: bash
    environment:
      CI_REPO_PATH: /absolute/path/to/project/repo
```

`CI_REPO_PATH` must be the absolute checkout path on the Woodpecker agent host. The folder must contain `Code/CI_CD/Steps/`.

Quick check:

```bash
test -f /absolute/path/to/project/repo/Code/CI_CD/Steps/01_Install.sh
```

The template intentionally uses:

```yaml
CI_REPO_PATH: __SET_ABSOLUTE_REPO_PATH__
```

Leave it broken until the real checkout path is deliberately entered.

## `Code/CI_CD/Config/env.env`

Set the project deployment values:

```env
CI_DEPLOY_DOMAIN="example.com"
CI_DEPLOYMENT_ROOT="/home/javed/Desk/Zetun_Web/10_Web_Runtime/Web_Layer/20_Static_Websites/10_Public_Facing"
CI_DEPLOY_HOST="g12"
CI_DEPLOY_KEEP_RELEASES="5"
```

Confirm the shared tool paths:

```env
CI_COMPRESSOR_BIN="/path/to/compressor"
CI_OG_IMAGE_DIR="/path/to/Og_Image"
```

`CI_DEPLOY_KEEP_RELEASES` is how many release folders stay on the host for rollback. It is required; the rule behind it and the order of pruning are in [Release layout, retention and rollback](../../Architecture/release_Layout.md).

The values come from the `web-comp-tools` skill's `path_File.md` (`Compressor_Bin` for `CI_COMPRESSOR_BIN`, `Og_Image_Home` for `CI_OG_IMAGE_DIR`). The project's `env.env` keeps its own copy, because the pipeline has to work without that skill installed; when a tool folder moves, update the copy.

The template intentionally has broken syntax at `CI_DEPLOY_DOMAIN`. Complete it before running the pipeline.

## Do Not Define Woodpecker Built-Ins

Do not add these to `Code/CI_CD/Config/env.env` or `.woodpecker.yml`:

```text
CI_COMMIT_BRANCH
CI_COMMIT_SHA
CI_PIPELINE_NUMBER
CI_PIPELINE_EVENT
CI_REPO
CI_REPO_URL
CI_WORKSPACE
```

Woodpecker provides these values at runtime. The local backend also protects `CI_WORKSPACE` from being overwritten, so trying to use it as the checkout path will not work.

## Verification

From the repo root:

```bash
rg -n "CI_REPO_PATH" .woodpecker.yml
test -f "$(rg -o 'CI_REPO_PATH:\\s*.*' .woodpecker.yml | sed 's/.*CI_REPO_PATH:\\s*//')/Code/CI_CD/Steps/01_Install.sh"

source Code/CI_CD/Config/env.env
test -n "$CI_DEPLOY_DOMAIN"
test -d "$CI_DEPLOYMENT_ROOT"
test -x "$CI_COMPRESSOR_BIN"
test -f "$CI_OG_IMAGE_DIR/src/main.js"
```
