# Hk

hk is a git hook runner by jdx — same author as mise and fnox. Defines hooks per-repo in `hk.pkl` (Apple's Pkl config language). Primary use in this repo: run a script automatically when specific files change at a git event — `post-merge` to redeploy or reinstall after a pull, `pre-commit` to validate before committing. Works with any git client-side hook (`post-merge`, `post-checkout`, `pre-commit`, `pre-push`, `commit-msg`, and more). Can also run linters in parallel with file-level locking, but that's optional.

For first-time setup, see [installation.md](installation.md).

## Two install modes

| Mode | Command | When |
|---|---|---|
| **Global** (recommended) | `hk install --global` | Git 2.54+ — installs config-based hooks into `~/.gitconfig`. Every repo with `hk.pkl` "just works"; repos without one are silent no-ops. |
| **Per-repo** (fallback) | `hk install` (in repo) | When Git 2.54+ isn't viable or you want hk scoped to specific repos. Falls back to script shims in `.git/hooks/` on older Git. |

> Don't run both. If global is active, per-repo install on top fires hooks **twice per event** because Git aggregates `hook.<name>.command` across scopes.

## Core commands

| Command | What it does |
|---|---|
| `hk init` | Generate `hk.pkl` in the current repo |
| `hk init --mise` | Same, plus a `mise.toml` with hk pinned and a `pre-commit` task |
| `hk validate` | Validate `hk.pkl` syntax. Useful in CI and after edits. |
| `hk run <hook>` | Run a hook explicitly without going through git (e.g. `hk run pre-commit`) |
| `hk check` | Run all "check" steps against modified files (read-only) |
| `hk check --all` | Lint the entire repo (CI mode) |
| `hk check --from-ref main` | Lint files changed since `main` |
| `hk fix` | Run "fix" steps (modify files to resolve issues) |
| `hk config dump` | Show effective merged configuration |
| `hk config sources` | Show where each setting came from |

## hk.pkl essentials

The first line is always the `amends` URL — it pins the Pkl schema version hk uses to evaluate this file. The schema version and the binary version (installed via mise) are independent: bumping the URL gets new config features and Builtins, but doesn't change which binary runs.

### Task trigger (primary use case)

Run a script on `post-merge` when specific files changed — no Builtins import needed:

```pkl
amends "package://github.com/jdx/hk/releases/download/v1.45.0/hk@1.45.0#/Config.pkl"

hooks {
  ["post-merge"] {
    steps {
      ["redeploy"] {
        glob = List("Code/**/*.go", "Code/go.mod", "Code/go.sum")
        check = "bash Code/deploy/redeploy.sh"
      }
    }
  }
}
```

- `glob` — which files trigger this step. If none of those files changed in the merge, the step is skipped silently.
- `check` — the command to run. Non-zero exit fails the hook.
- Multiple steps run in parallel by default; use `exclusive = true` on a step to block others until it finishes.

### Linting (optional, when needed)

For pre-commit linting, also import Builtins and add `fix`/`stash`:

```pkl
amends "package://github.com/jdx/hk/releases/download/v1.45.0/hk@1.45.0#/Config.pkl"
import "package://github.com/jdx/hk/releases/download/v1.45.0/hk@1.45.0#/Builtins.pkl"

hooks {
    ["pre-commit"] {
        fix = true       // apply linter fixes
        stash = "git"    // stash unstaged work, restore via 3-way merge
        steps {
            ["prettier"] = Builtins.prettier
            ["eslint"] {
                glob = List("*.js", "*.ts")
                check = "eslint {{files}}"
                fix = "eslint --fix {{files}}"
            }
        }
    }
}
```

## Bypass and skip mechanisms

| Goal | How |
|---|---|
| Bypass all hooks for one command | `HK=0 git commit -m "..."` |
| Disable hk in one repo permanently | `git config --local hook.hk-<event>.enabled false` (one entry per event) |
| Skip specific steps in one repo | `git config --local hk.skipSteps "slow-linter,noisy"` |
| Skip steps for one run | `HK_SKIP_STEPS=slow-linter hk run pre-commit` |
| Skip an entire hook | `git config --local hk.skipHook "pre-push"` or `HK_SKIP_HOOK=pre-push` |
| Replace project hooks locally without committing | Create `hk.local.pkl` that `amends "./hk.pkl"` and overrides hooks (gitignore it) |

## mise integration

Set `HK_MISE=1` so hk wraps hooks with `mise x`. This guarantees the tools defined in `mise.toml` are on PATH when hooks run, even if a teammate hasn't activated mise in their shell.

Recommended `mise.toml` pattern for an hk project:

```toml
[tools]
hk = "latest"
pkl = "latest"
# plus the actual linters: prettier, eslint, etc.

[env]
HK_MISE = 1

[hooks]
postinstall = "hk install --mise"   # auto-install hooks on `mise install`
```

With this, a teammate running `mise install` after cloning gets hooks wired automatically — no per-repo `hk install` step, even without the global setup.

## Troubleshooting

| Issue | Fix |
|---|---|
| Hooks fire twice per commit | You ran both `hk install --global` and `hk install` per-repo. Pick one. To keep global only: `git config --local hook.hk-<event>.enabled false` for each event in the repo's local config. |
| `hk install --global` says config-based hooks not supported | Git is older than 2.54. Upgrade, or fall back to per-repo `hk install` (shim mode). |
| Tools not found inside hooks (script or linter not on PATH) | Set `HK_MISE=1` so hk wraps hooks with `mise x`, or ensure the tool is on the system PATH. |
| Need to commit through a broken hook once | `HK=0 git commit ...` |
| Don't want pkl CLI on the machine | Set `HK_PKL_BACKEND=pklr` to use hk's built-in Rust evaluator. |
| Unsure why a step was skipped or ran | `hk config dump` for the merged config; `hk config sources` to see where each setting came from. |

## References

- [Installation](installation.md): one-time setup playbook — install via mise, Git 2.54+ check, global vs per-repo install, project `hk init`, optional `mise.toml` integration with `HK_MISE=1`. Also contains the clone command for the upstream hk docs (`Repos/Tool_Manag/hk/`) when deeper reference is needed.
