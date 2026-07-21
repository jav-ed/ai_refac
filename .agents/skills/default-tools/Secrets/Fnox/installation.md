# Fnox — Installation and Key Setup

## Install fnox

Preferred method is via mise:

```bash
mise use -g fnox
fnox --version
```

## Configure decryption (per machine)

Generate a native age key — do this once per machine:

```bash
age-keygen -o ~/.config/fnox/age.txt
chmod 600 ~/.config/fnox/age.txt
```

Fnox auto-discovers the key at `~/.config/fnox/age.txt`. No env var needed. The `age1...` public key printed by `age-keygen` is what goes into `recipients`.

Verify everything is wired up:

```bash
fnox doctor
```

## Known recipients (our devices)

These are the native age public keys for the two managed devices. Any new `fnox.toml` should include both so secrets are decryptable on either machine.

```toml
[providers]
age = { type = "age", recipients = [
    "age1uy9ps3p4460de20v8fgvt6gyg5ml0wscesd32m4693567aaumgnsspt8x5",  # jav (local)
    "age1areaucgxqcwvnsnr3mpz46g27pc9gf8qqcvaaa3k5fn3fluvde3s27cql9",  # g12 (server)
] }
```

## Initialize a new project

```bash
cd your-project
fnox init
```

Then replace the generated `[providers.age]` block with the known recipients above.

## Adding a new recipient (new machine or server)

1. On the new machine, generate a native age key: `age-keygen -o ~/.config/fnox/age.txt` — it prints the `age1...` public key
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
git clone --depth 1 https://github.com/jdx/fnox.git Repos/Tool_Manag/fnox
```

If `Repos/` doesn't exist yet, create it and add it to `.gitignore` first:

```bash
mkdir -p Repos
echo "Repos/" >> .gitignore
```

Docs are under `Repos/Tool_Manag/fnox/docs/`:
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
