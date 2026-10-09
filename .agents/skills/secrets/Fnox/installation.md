# Fnox: Installation and Key Setup

## Install fnox

Fnox is installed as a shared mise system tool (tool name `fnox`) and linked at `/usr/local/bin/fnox`; the `default-tools` skill describes the install procedure. Verify:

```bash
fnox --version
```

## Configure decryption (per machine)

Create the machine's age key and the symlinks once, as described in [Age keys and recipients](../Age/keys_And_Recipients.md). Fnox auto-discovers the key at `~/.config/fnox/age.txt`; no env var is needed. The `age1...` public key printed by `age-keygen` is what goes into `recipients`.

Verify everything is wired up:

```bash
fnox doctor
```

## Known recipients (our devices)

Any new `fnox.toml` should include the public keys of both managed devices so secrets are decryptable on either machine. The ready-to-paste `[providers]` block lives in [Age keys and recipients](../Age/keys_And_Recipients.md).

## Initialize a new project

```bash
cd your-project
fnox init
```

Then replace the generated `[providers.age]` block with the block from [Age keys and recipients](../Age/keys_And_Recipients.md).

## Adding a new recipient (new machine or server)

1. On the new machine, create the age key and symlinks ([Age keys and recipients](../Age/keys_And_Recipients.md)); `age-keygen` prints the `age1...` public key
2. Add that `age1...` public key to `recipients` in `fnox.toml`
3. Re-encrypt all secrets so the new key can decrypt:
   ```bash
   fnox reencrypt -p age
   ```
4. Commit `fnox.toml`

## Removing a recipient

1. Remove their key from `recipients`
2. Re-encrypt:
   ```bash
   fnox reencrypt -p age
   ```
3. Commit `fnox.toml`

## Full upstream docs (when more detail is needed)

Clone the fnox source repo into `Repos/` at the repo root. `Repos/` is gitignored, so the clone stays local.

```bash
# From the repo root
git clone --depth 1 https://github.com/jdx/fnox.git Repos/fnox
```

If `Repos/` doesn't exist yet, create it and add it to `.gitignore` first:

```bash
mkdir -p Repos
echo "Repos/" >> .gitignore
```

Record the clone in the project's `Project_Manag/Docs/Setup/repos_List.md` (doc-start convention) so it can be restored on another machine.

Docs are under `Repos/fnox/docs/`:
- `guide/` — quick-start, how-it-works, profiles, hierarchical config, shell integration, leases
- `cli/` — per-command reference (get, set, exec, list, export, etc.)
- `providers/` — provider-specific setup (`age.md` is ours)
- `reference/` — full config and environment variable reference

## First secret

```bash
fnox set DATABASE_URL "postgresql://localhost/mydb"
fnox get DATABASE_URL                        # verify decryption
fnox exec -- env | grep DATABASE_URL         # verify injection
```

Then open `fnox.toml` and add a comment above the line (see Comments rule in [linker_Fnox.md](linker_Fnox.md)).
