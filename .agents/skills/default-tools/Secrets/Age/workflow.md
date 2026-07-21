# Raw Age — File-Level Encryption

> **Default for new work is [sops](../Sops/workflow.md), not this.** Use raw age when sops doesn't fit — primarily for **TOML files** (sops doesn't support TOML), or when you want a simple whole-file encryption pattern with no per-field metadata. The Dynaconf workflow below is the canonical example.

Use this pattern when a framework (Dynaconf, dotenv, config parser) needs to read a structured secrets file from disk. Age encrypts the whole file; the framework reads the plaintext normally after decryption.

This repo's secrets live in `Input/Dynaconf/Secrets/` and are managed by `Input/Dynaconf/manage_secrets.sh`.

## Mental model

```
plaintext .secrets.toml   →  age encrypt  →  .secrets.toml.age   (committed to git)
.secrets.toml.age         →  age decrypt  →  .secrets.toml        (gitignored, read by Dynaconf)
```

Decrypt once at startup. The app never calls age again — it reads the plaintext file like any other config.

## Commands

```bash
# Decrypt to disk (safe to run repeatedly — overwrites plaintext)
./Input/Dynaconf/manage_secrets.sh decrypt

# Decrypt only what is missing (idempotent — use this at startup / in scripts)
./Input/Dynaconf/manage_secrets.sh ensure

# Encrypt plaintext back to .age (after editing secrets)
./Input/Dynaconf/manage_secrets.sh encrypt
```

`ensure` is the safe default for CI and startup scripts: it skips files that are already decrypted and fails loudly if a `.age` file is missing entirely.

## Identity (decryption key)

The canonical age key lives at `~/.config/age/age.txt`. Both fnox and sops symlink to it. The script reads from `$AGE_IDENTITY`, falling back to `~/.config/age/age.txt`.

```bash
# Check which identity is active
echo ${AGE_IDENTITY:-~/.config/age/age.txt}

# Override for this session
export AGE_IDENTITY=~/.config/age/age.txt
```

## Recipients (encryption targets)

Two devices can decrypt: the local machine (`jav`) and the server (`g12`). Their native age public keys are hardcoded in the script:

```
jav (local):  age1uy9ps3p4460de20v8fgvt6gyg5ml0wscesd32m4693567aaumgnsspt8x5
g12 (server): age1areaucgxqcwvnsnr3mpz46g27pc9gf8qqcvaaa3k5fn3fluvde3s27cql9
```

Keys live at `~/.config/age/age.txt` on each device. `~/.config/fnox/age.txt` and `~/.config/sops/age/keys.txt` are symlinks to it.

## Editing secrets

1. Decrypt: `./Input/Dynaconf/manage_secrets.sh decrypt`
2. Edit the plaintext `.toml` file in `Input/Dynaconf/Secrets/`
3. Re-encrypt: `./Input/Dynaconf/manage_secrets.sh encrypt`
4. Commit the `.age` file. The plaintext is gitignored and must never be committed.

## Adding a new recipient

1. Get the new machine's native age public key (e.g. from `~/.config/fnox/age.txt` on that machine)
2. Add it to the `RECIPIENTS` array in `manage_secrets.sh`
3. Decrypt then re-encrypt so the new key is included:
   ```bash
   ./Input/Dynaconf/manage_secrets.sh decrypt
   ./Input/Dynaconf/manage_secrets.sh encrypt
   ```
4. Commit `manage_secrets.sh` and the updated `.age` files

## Files managed

| Ciphertext (committed) | Plaintext (gitignored) | Purpose |
|---|---|---|
| `.secrets.toml.age` | `.secrets.toml` | Platform API tokens, credentials |
| `.ai_Secrets.toml.age` | `.ai_Secrets.toml` | AI API keys (OpenAI, Anthropic, etc.) |
