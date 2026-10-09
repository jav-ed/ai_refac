# Mise

Use mise only to install and update selected system tools. Normal commands never depend on mise activation, shims, or project configuration: they follow plain filesystem links. Just owns repository tasks; mise is not a task runner.

## Install method, in order of preference

1. The distribution package, when it provides the required version.
2. The tool's official installer for self-contained tools such as Bun and Oh My Zsh.
3. A mise system install, when the distribution package is too old and mise has a reliable backend.
4. A documented source build, when the options above are unsuitable.

## Layout

```text
/usr/local/bin/jq
  -> /usr/local/share/mise/installs/jq/latest/jq
  -> /usr/local/share/mise/installs/jq/1.8.2/jq
```

Tools install under `/usr/local/share/mise/installs`, and each approved executable is linked into `/usr/local/bin`. The shared tool list is the `[tools]` table of `/etc/mise/config.toml`, an installed copy of a manifest: one line per tool, `name = "latest"`, with a short purpose comment. When a repository tracks the manifest, edit the tracked file and reinstall it instead of editing `/etc/mise/config.toml` by hand:

```bash
sudo install -D -o root -g root -m 0644 <tracked-manifest.toml> /etc/mise/config.toml
```

## Install (needs root)

First machine only: install mise itself.

```bash
curl https://mise.run | sh
sudo install -m 0755 ~/.local/bin/mise /usr/local/bin/mise
```

Then install everything in the manifest. Run from `/` with a clean environment so no user config leaks in:

```bash
cd / && sudo env -i HOME=/root USER=root PATH=/usr/local/bin:/usr/bin:/bin \
  MISE_INSTALLS_DIR=/usr/local/share/mise/installs \
  /usr/local/bin/mise install --system --yes
sudo chmod -R a+rX /usr/local/share/mise
```

The `chmod` is required because g12 uses umask `077`.

## Link executables

Package layouts vary, so link each approved command explicitly instead of exposing whole packages. The target must exist and be executable. Where a release folder is nested, locate the executable with `find -H /usr/local/share/mise/installs/<tool>/latest -type f -perm /111` and continue only when exactly one match is found.

```bash
sudo ln -sfn /usr/local/share/mise/installs/<tool>/latest/<path-to-executable> /usr/local/bin/<command>
```

## Update

Reinstall the manifest (as above), then upgrade. `MISE_INSTALLS_DIR` stays explicit because `mise upgrade` has no `--system` option. The upgrade prunes replaced versions; finish by normalizing permissions and refreshing the links.

```bash
cd / && sudo env -i HOME=/root USER=root PATH=/usr/local/bin:/usr/bin:/bin \
  MISE_INSTALLS_DIR=/usr/local/share/mise/installs \
  /usr/local/bin/mise upgrade
sudo chmod -R a+rX /usr/local/share/mise
```

## Verify

Check each exposed tool without user startup files or mise variables, then as an unprivileged account:

```bash
env -i PATH=/usr/local/bin:/usr/bin:/bin /usr/local/bin/jq --version
sudo -u nobody env -i PATH=/usr/local/bin:/usr/bin:/bin /usr/local/bin/jq --version
```

## Boundaries

- Do not add mise shims to the system `PATH`, require `mise activate` on servers, or point a link at a shim.
- Do not use mise tasks or a project-level `mise.toml` for version lookup.
- Do not expose every file of an installed package; link only the approved commands.
- Add each new shared tool to the manifest with a purpose comment, then install and link it.
