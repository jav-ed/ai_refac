# Mise — Shared System-Wide Installation

Server-side playbook for setting up `/opt/mise` with multi-user access — admin (`javed`) installs/manages tools, service users (`caddy`, `stalwart`, etc.) read and execute. Runs once per server. For a local dev install, just run the official installer (`curl https://mise.run | sh`); this file is the production server pattern.

## Prerequisites

- Root/sudo access
- Admin user (`javed`) created
- Service users either already exist or will be created later

## Step 1 — Install the binary globally

Move the binary to a system-wide PATH location so all users can find `mise`.

```bash
# After running the official installer to ~/.local/bin:
sudo mv ~/.local/bin/mise /usr/local/bin/mise

# Verify
which mise
# /usr/local/bin/mise
```

## Step 2 — Create the shared data directory

```bash
sudo mkdir -p /opt/mise
sudo chown -R javed:javed /opt/mise
sudo chmod -R 755 /opt/mise
```

This lets `javed` install tools without sudo while service users like `caddy` keep read+execute access.

## Step 3 — Internal mise config

Tells mise where to store its data without relying on shell environment.

```bash
sudo mkdir -p /etc/mise
```

Create `/etc/mise/config.toml`:

```toml
[env]
MISE_DATA_DIR = "/opt/mise"
MISE_CONFIG_DIR = "/opt/mise/config"
MISE_CACHE_DIR = "/opt/mise/cache"
MISE_STATE_DIR = "/opt/mise/state"

[tools]
# Optional — pin global tools here if desired
```

## Step 4 — System-wide PATH and env

Edit `/etc/environment` so all processes (cron, systemd, services) see mise's tools without shell activation:

```bash
sudo nano /etc/environment
```

Add:

```
MISE_DATA_DIR="/opt/mise"
PATH="/opt/mise/shims:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin"
```

`/opt/mise/shims` must be the **first** entry in PATH. Changes require logout/login to take effect for interactive shells.

## Step 5 — User shell config: do nothing

We deliberately do **not** add `eval "$(mise activate zsh)"` to `.zshrc` / `.bashrc`. The server pattern relies entirely on the static PATH in `/etc/environment`. This guarantees background services (cron, systemd) see exactly the same tool versions as interactive shells, with zero shell-startup overhead.

## Step 6 — Verify

As `javed`:

```bash
# Re-login first, or `source /etc/environment` for testing
mise install bun@latest
mise where bun
# /opt/mise/installs/bun/<version>
```

As a service user:

```bash
sudo -u nobody env | grep MISE
# MISE_DATA_DIR=/opt/mise

sudo -u caddy /opt/mise/shims/bun --version
# bun's version output
```

## Updating mise itself

```bash
sudo mise self-update
```

Sudo is required because the binary lives in `/usr/local/bin`.

## Why no shell activation hooks

We avoid `eval "$(mise activate ...)"` on the server intentionally:

1. **Performance** — zero overhead during shell startup.
2. **Predictability** — cron, systemd, and interactive shells all resolve the same tool versions through the static PATH.
3. **Stability** — no dynamic path manipulation that could conflict with scripts or remote execution.

Tradeoff: env vars defined in `[env]` of any `mise.toml` are **not** injected into the shell, only into processes started via shims or `mise exec`. For env vars that must be globally visible on the server, use `/etc/environment` instead.

## pyinfra notes

For future automation:

| Step | pyinfra operation |
|---|---|
| Install binary | `files.download` to `/usr/local/bin/mise` + `chmod +x` |
| Create data dir | `files.directory` `/opt/mise` (user=`javed`, mode=`755`) |
| Internal config | `files.put` content to `/etc/mise/config.toml` |
| System config | `files.line` or `files.put` to `/etc/environment` |
