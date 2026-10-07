# Install and verify eza

Use this file when `eza` is missing or its installed build cannot run the skill's commands. Assume installation is complete during ordinary file-tree work. Confirm the host, operating system, architecture, and existing tool manager before choosing an installation method.

The optional structural counter uses Python 3's standard library and Git; it needs no Python packages. Refac setup and language prerequisites belong to the separate `refac-cli` skill.

## Check before installing

```bash
command -v eza
eza --version
```

If the executable already supports the required commands, use it. A new audit does not require an upgrade. An alias, missing PATH entry, or incompatible build needs diagnosis rather than a second installation that hides the first.

## Choose the machine's installation method

Follow the established package or shared-tool policy. On Arch/Manjaro, where the distribution package is suitable, install it with:

```bash
sudo pacman -S --needed eza
```

Use existing package databases or the machine's normal update procedure; do not introduce a partial system upgrade. For a machine that manages eza through the shared mise baseline, update the owning manifest and approved executable link, then use its installation recipe. Avoid shell activation, shims, and project-local tool versions for a shared system command.

- [Default tools](../../default-tools/SKILL.md): the shared mise acquisition policy and navigation to the owning system-tool installation procedures.
- [Upstream installation guide](https://github.com/eza-community/eza/blob/main/INSTALL.md): supported distribution packages and official binary installation for other operating systems.

## Official binary for a user installation

When installation is authorized for the current user and system-wide installation is unavailable, use an official release in a user-owned location. State the installation scope explicitly.

1. Read the selected release's asset metadata and choose the archive for the host's architecture and runtime. Do not assume that an x86_64 Linux archive fits every machine.
2. Download the versioned archive from the official repository into a temporary directory. Compare its SHA-256 with the release asset's published digest before executing it; fail with the expected and actual hashes if they differ.
3. Extract the executable into a versioned directory such as `~/.local/share/eza/<version>/bin/eza`, and give it executable permissions. Inspect existing destinations first; do not overwrite an unrelated installation.
4. Expose that executable with a link at `~/.local/bin/eza`. Confirm that directory is on the intended user's PATH. Record the source URL, version, digest, and installed path beside the installation.
5. Run the verification below. For a future upgrade, verify the replacement before changing the active link.

- [Official releases](https://github.com/eza-community/eza/releases): release tags, architecture-specific binary archives, and asset metadata for manual installation.

On `javPc`, verified on 2026-09-06: eza `0.23.5` with Git support is installed for `jav`. `~/.local/bin/eza` points to `~/.local/share/eza/0.23.5/bin/eza`; `installation.json` beside `bin/` records the verified archive digest and source. This used the official x86_64 Linux musl archive because interactive sudo authentication was unavailable. This is an installation record, not a version pin for other machines.

## Verify the actual command

```bash
command -v eza
eza --version
eza --tree --level=1 --git-ignore --group-directories-first \
  --color=never --icons=never --hyperlink=never -- .
```

Run the tree command inside a Git checkout. Verify that known ignored entries are absent; accepting a flag alone does not prove the expected filtering. Git filtering requires a build with Git support. Keep command errors visible and resolve them before relying on the output.

For the user installation above, verify execution without shell initialization:

```bash
env -i PATH="$HOME/.local/bin:/usr/local/bin:/usr/bin:/bin" eza --version
```

- [Everyday use](../SKILL.md): return to the regular inspection commands once setup is verified.
