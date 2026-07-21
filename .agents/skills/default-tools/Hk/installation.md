# Hk — Installation

hk is installed via mise (the same way fnox is). Setup has two layers: install the binary once per machine, then choose between global hooks (recommended) or per-repo hooks.

## Prerequisites

- mise installed (see `default-tools/Mise/installation.md` for first-time mise setup).
- For the recommended global install: **Git 2.54+** (`git --version`). Older Git can't read config-based hooks (`hook.<name>.command`), which is the mechanism that makes `hk install --global` work without touching every repo.
- If your distro doesn't ship Git 2.54+, build from source. The global install is the whole point of hk's setup ergonomics — without it you fall back to running `hk install` once per repo.

## Step 1 — Install the binary

```bash
mise use -g hk@latest
hk --version
```

Optional companion (only if you'll write or evaluate `hk.pkl` files yourself with the standalone CLI):

```bash
mise use -g pkl@latest
```

If you don't want the pkl CLI on your machine, export `HK_PKL_BACKEND=pklr` — hk uses its built-in Rust evaluator instead.

## Step 2a — Global hook install (recommended)

Runs once per machine. After this, every repo with an `hk.pkl` automatically picks up its hooks. Repos without one are silent no-ops.

```bash
hk install --global
```

This writes `hook.hk-<event>.command` entries into `~/.gitconfig` for: `pre-commit`, `pre-push`, `commit-msg`, `prepare-commit-msg`, `post-checkout`, `post-merge`, `post-rewrite`, `pre-rebase`, `post-commit`.

To remove:

```bash
hk uninstall --global
```

## Step 2b — Per-repo install (fallback)

Use when Git 2.54+ isn't available or you want hk scoped to specific repos:

```bash
cd <project-with-hk.pkl>
hk install
```

On Git 2.54+ this writes `git config --local hook.hk-<event>.command`. On older Git it falls back to script shims in `.git/hooks/`. Pass `--legacy` to force shim mode regardless of Git version.

> Do not run per-repo install on top of the global install in the same repo — Git aggregates `hook.<name>.command` across scopes and hooks fire twice. Pick one. To disable global in a single repo: `git config --local hook.hk-<event>.enabled false` for each event you care about.

## Step 3 — Project setup

In a fresh repo, generate `hk.pkl`:

```bash
hk init
```

Or, with mise integration baked in (recommended for team consistency):

```bash
hk init --mise
```

The `--mise` form also generates a `mise.toml` pinning hk and adding a `pre-commit` task so anyone can run `mise run pre-commit` as an alias for `hk run pre-commit`.

For a `mise.toml` already present, the recommended pattern:

```toml
[tools]
hk = "latest"
pkl = "latest"
# plus the actual linters

[env]
HK_MISE = 1            # wrap hooks with `mise x` so tool PATH is set up

[hooks]
postinstall = "hk install --mise"   # auto-install hooks on `mise install`
```

A teammate running `mise install` after cloning then gets hooks wired automatically — no per-repo `hk install` even without the global setup.

## Step 4 — Verify

```bash
hk --version
hk run pre-commit                # runs the pre-commit hook explicitly
hk config dump                   # show effective configuration
hk config sources                # show where each setting came from
```

## Manual gitconfig (alternative to `hk install --global`)

If you'd rather wire hooks by hand:

```ini
# ~/.gitconfig
[hook "hk-pre-commit"]
    command = test "${HK:-1}" = "0" || hk run pre-commit --from-hook "$@"
    event = pre-commit
[hook "hk-pre-push"]
    command = test "${HK:-1}" = "0" || hk run pre-push --from-hook "$@"
    event = pre-push
[hook "hk-commit-msg"]
    command = test "${HK:-1}" = "0" || hk run commit-msg --from-hook "$@"
    event = commit-msg
```

The `--from-hook` flag tells hk to exit silently when no `hk.pkl` is present. The `test "${HK:-1}" = "0" ||` prefix is the bypass — `HK=0 git commit` skips hooks for one command.

If hk is managed via mise without auto-activation, replace `hk` with `mise x -- hk` in those `command =` lines.

## Full upstream docs

If detail beyond this skill is needed, the upstream hk docs cover everything. Clone into the gitignored `Repos/` folder at the repo root:

```bash
# From the repo root
git clone --depth 1 https://github.com/jdx/hk.git Repos/Tool_Manag/hk
```

Read order: `Repos/Tool_Manag/hk/docs/getting_started.md`, then `configuration.md`, `hooks.md`, and `mise_integration.md`.
