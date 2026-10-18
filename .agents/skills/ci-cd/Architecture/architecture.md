# Architecture

How the pipeline works and why it is shaped this way. Read once to understand the assumptions; refer back when something feels wrong. The static-site path remains the default; compiled Rust utilities use the same ownership boundaries with different step scripts.

## Contents

- [Pipeline shape](#pipeline-shape)
- [Local backend](#why-local-backend)
- [Script ownership](#why-scripts-own-execution)
- [Just boundary](#justfile-boundary)
- [Non-interactive tools](#non-interactive-tool-environments)
- [`skip_clone`](#skip_clone-true)
- [Releases](#releases)
- [Footguns](#footguns)

## Pipeline Shape

The project keeps tool and task entrypoints at the repo root, while the CI/CD implementation lives under `Code/CI_CD/`.

Root files:

```text
.woodpecker.yml
justfile
```

Implementation:

```text
Code/CI_CD/
|-- Config/
|   `-- env.env
|-- Lib/
`-- Steps/
```

Woodpecker calls `Code/CI_CD/Steps/*.sh` directly. Each step sources `Code/CI_CD/Lib/runtime.sh`, and runtime loads `Code/CI_CD/Config/env.env`. Just provides the mandatory local shortcut recipes that invoke the same step scripts.

| Task | Step script | What it does |
|------|-------------|--------------|
| `01_Install` | `Code/CI_CD/Steps/01_Install.sh` | `bun install` |
| `02_Build` | `Code/CI_CD/Steps/02_Build.sh` | `bun run build` (Astro -> `dist/`) |
| `03_Og_Images` | `Code/CI_CD/Steps/03_Og_Images.sh` | Calls the external OG generator with `--project_Root "$CI_PROJECT_DIR"`. The cards are build output: ignored by git, written into `dist/`, copied to the host with it (the `web-comp-tools` skill, Og_Image page) |
| `04_Compress` | `Code/CI_CD/Steps/04_Compress.sh` | Runs the shared compressor over `dist/` |
| `05_Deploy` | `Code/CI_CD/Steps/05_Deploy.sh` | Checks target shape, rsyncs `dist/` to a release directory, and swaps the active `dist` symlink |

What each tool in these steps does and how it fails is described in the `web-comp-tools` skill.

`.woodpecker.yml` sets `CI_REPO_PATH` and calls each step script through that absolute checkout path. `Code/CI_CD/Lib/runtime.sh` derives `CI_PROJECT_DIR` from the script location, loads `Code/CI_CD/Config/env.env`, and provides required-variable checks.

## Why Local Backend

Woodpecker can run pipelines in containers or directly on the host. We use the host through the local backend for three reasons:

1. No container startup overhead.
2. Direct filesystem access to the working checkout and `dist/`.
3. Native host binaries for Bun, the compressor, the OG generator, SSH, and rsync.

The tradeoff is host coupling: the agent must run on the machine where this checkout and the shared tools exist.

## Why Scripts Own Execution

Woodpecker should stay small. It only starts the project scripts through `CI_REPO_PATH`:

```yaml
commands:
  - "$CI_REPO_PATH/Code/CI_CD/Steps/02_Build.sh"
```

The scripts own:

- project env loading,
- repo-root detection,
- deploy safety checks,
- calls to Bun, OG generation, compression, and deployment.

Most project variables stay in one place: `Code/CI_CD/Config/env.env`.

Exception: `CI_REPO_PATH` lives in `.woodpecker.yml`, because Woodpecker starts in a temporary workspace. The pipeline must know the real checkout path before it can start any repo-owned script. Once a script starts, `runtime.sh` derives the repo root from its own location.

## Justfile Boundary

`justfile` is intentionally dumb in this design. It defines local recipes that only call the matching `Code/CI_CD/Steps/*.sh` scripts:

```just
Step_02_Build:
    Code/CI_CD/Steps/02_Build.sh
```

This keeps local commands comfortable without making CI depend on a task runner's environment loading, tool installation, shell startup behavior, or home-directory assumptions.

Do not move project env loading, deploy safety checks, shared-tool paths, or Woodpecker-specific decisions into `justfile`. Those belong in `.woodpecker.yml` or `Code/CI_CD/`.

When the command surface grows, the root `justfile` may import focused files under `Code/Just/`. This improves command grouping without changing ownership: recipe files still only call `Code/CI_CD/Steps/*.sh` or compose other local recipes.

## Non-Interactive Tool Environments

The local backend does not guarantee the same `HOME`, shell initialization, PATH, or version-manager defaults as an interactive terminal. A local command succeeding through an implicit Rustup default does not prove that Woodpecker can select Cargo.

Compiled Rust pipelines must keep these project-owned values in `Code/CI_CD/Config/env.env`:

- exact `CI_RUST_TOOLCHAIN`
- absolute `CI_RUSTUP_BIN`
- absolute `CI_RUSTUP_HOME`
- absolute `CI_CARGO_HOME`
- `CI_BINARY_NAME`, installation host, and absolute installation path

The runtime must export the configured homes and invoke tools through `rustup run <exact-toolchain>`. Missing paths or toolchains are hard errors; the pipeline must not download or substitute a toolchain implicitly. Prove the boundary locally with an isolated temporary HOME before the first push. See [Rust binary tool](../Application/Integration/rust_Binary_Tool.md).

## `skip_clone: true`

The pipeline starts with:

```yaml
skip_clone: true
```

`skip_clone` only disables Woodpecker's automatic clone step. It does not make the local backend start in the existing checkout. The local backend still creates a temporary workspace for each workflow.

Therefore every step must call the real checkout script by absolute path:

```yaml
environment:
  CI_REPO_PATH: /absolute/path/to/project/repo

commands:
  - "$CI_REPO_PATH/Code/CI_CD/Steps/02_Build.sh"
```

Implications:

- the agent machine must have the checkout,
- `.woodpecker.yml` must contain the correct absolute `CI_REPO_PATH` for that checkout,
- uncommitted local changes can affect manual reruns,
- caches such as `node_modules/` and `.astro/` persist across runs.

## Releases

Deploy writes every build into its own dated release folder and activates it by switching the `main/dist` symlink that Caddy serves, so a deploy is atomic, needs no Caddy reload, and keeps older releases for rollback. The layout, naming, activation steps, retention count and rollback are described once in [Release layout, retention and rollback](release_Layout.md), together with what a repository may override.

## Footguns

### Host Coupling

Local backend CI is tied to this checkout and this host's tools. A multi-worker setup needs a different design.

### Uncommitted Changes

Because clone is skipped, a manual rerun can pick up uncommitted files from disk.

### Template Safety

The template intentionally leaves `CI_REPO_PATH` as `__SET_ABSOLUTE_REPO_PATH__` in `.woodpecker.yml` and has broken syntax for `CI_DEPLOY_DOMAIN` in `Code/CI_CD/Config/env.env`. This is a deployment safety measure: copied templates cannot run until the checkout path and project domain are deliberately entered.

### External Tool Drift

The compressor and OG generator live outside the Astro repo. If either changes its CLI or output contract, update the `web-comp-tools` skill, then rerun the matching script locally before trusting a webhook run.
