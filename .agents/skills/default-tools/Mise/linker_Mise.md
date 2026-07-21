# Mise

Mise replaces language-specific version managers (nvm, pyenv, rustup, etc.) with a single tool, and adds task running and per-directory env vars. In this repo it runs in two modes:

- **Local/Dev**: standard install at `~/.local/share/mise`, activated via shell hooks (`eval "$(mise activate zsh)"` in `.zshrc`).
- **Server (`g12`)**: shared system-wide install at `/opt/mise`, **no shell hooks** — uses static `PATH` in `/etc/environment`.

For first-time setup of the shared server install, see [installation.md](installation.md).

## Core commands

| Command | What it does |
|---|---|
| `mise install` | Install all tools listed in `mise.toml` (run after cloning a repo) |
| `mise use <tool>@<ver>` | Install + pin a version in the current folder's `mise.toml` |
| `mise use --global <tool>@<ver>` | Install + set as the system default (admin only on server) |
| `mise current` | List currently active tools and the source of each version |
| `mise where <tool>` | Path to the tool's installation |
| `mise x <tool> -- <cmd>` | One-off run without installing globally |
| `mise x -C ./path -- <cmd>` | Run a command using another directory's `mise.toml` |
| `mise upgrade <tool>` | Upgrade an installed tool |
| `mise self-update` | Update the mise binary itself (sudo on server) |
| `mise trust` | Trust a `mise.toml` (mise refuses to read untrusted configs) |

> Mise walks up the directory tree to find `mise.toml`. Run from anywhere inside the repo and it resolves up.

## Local vs server differences

| | **Local (Standard)** | **Server (Robust)** |
|---|---|---|
| Binaries | `~/.local/share/mise/installs/` | `/opt/mise/installs/` |
| Global config | `~/.config/mise/config.toml` | `/etc/mise/config.toml` |
| Local config | `mise.toml` (project root) | `mise.toml` (project root) |
| Shims | `~/.local/share/mise/shims/` | `/opt/mise/shims/` |
| Activation | `eval "$(mise activate zsh)"` in `.zshrc` | None — static PATH in `/etc/environment` |

On the server, the admin user `javed` manages tools (write access); service users (`caddy`, `stalwart`, etc.) get read+execute only.

## Critical gotchas

### Env vars in `mise.toml` are not visible in the shell

Without shell hooks (the server case), variables defined in `[env]` of any `mise.toml` are injected only when running tools through a shim or `mise x`. The interactive shell itself never sees them.

```bash
# mise.toml has [env] DATABASE_URL = "postgres://..."

echo $DATABASE_URL                                  # → empty (shell unaware)
mise x -- printenv DATABASE_URL                     # → "postgres://..." (mise injects)
node -e 'console.log(process.env.DATABASE_URL)'     # → "postgres://..." (shim injects)
```

If a service or script needs an env var globally on the server, define it in `/etc/environment`, not in `mise.toml`. Mise on the server is treated as a tool-version manager, not an env-var distribution layer.

### Trust prompts

Mise refuses to read a `mise.toml` it has not been told to trust. After `git clone`, after editing `mise.toml`, or when changing into an unfamiliar project:

```bash
mise trust
```

### Local overrides global

If a tool seems "stuck" on the wrong version, walk up the directory tree looking for a rogue `mise.toml` — local always wins over global.

### Root vs user

- **Local**: never run mise as root. If permissions break: `chown -R $USER ~/.local/share/mise`.
- **Server**: only `javed` writes to `/opt/mise`; service users read-only. `mise self-update` needs `sudo` because the binary is in `/usr/local/bin`.

## Troubleshooting

| Error | Cause | Fix |
|---|---|---|
| `command not found: <tool>` | Shims dir not in PATH | Server: confirm `/opt/mise/shims` is in `/etc/environment`. Local: confirm shell hooks are active in `.zshrc` |
| Wrong version selected | Local `mise.toml` overriding global | `mise current` to see source; remove or fix the rogue file |
| `untrusted config file` | Mise hasn't trusted this `mise.toml` | Run `mise trust` from the directory |
| Permission denied | Wrong owner on data dir | Local: `chown -R $USER ~/.local/share/mise`. Server: `sudo chmod -R 755 /opt/mise` |
| Service can't find tool | Service env doesn't include shims dir | Add `/opt/mise/shims` to the service's `Environment` directive, or symlink the shim into `/usr/bin/<tool>` |

## References

- [Installation](installation.md): one-time playbook for the shared system-wide install on a new server (binary placement, `/opt/mise` ownership, `/etc/mise/config.toml`, `/etc/environment` PATH).
