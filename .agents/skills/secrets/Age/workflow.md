# Raw Age: File-Level Encryption

> **Default for new work is [sops](../Sops/workflow.md), not this.** Use raw age when sops does not fit: primarily for **TOML files** (sops does not support TOML), or when a simple whole-file encryption pattern with no per-field metadata is wanted.

Use this pattern when a framework (Dynaconf, dotenv, a config parser) needs to read a structured secrets file from disk. Age encrypts the whole file; the framework reads the plaintext normally after decryption. The machine key, the public keys of our devices and the steps for adding a device are in [Age keys and recipients](keys_And_Recipients.md).

## Mental model

```
plaintext .secrets.toml   →  age encrypt  →  .secrets.toml.age   (committed to git)
.secrets.toml.age         →  age decrypt  →  .secrets.toml        (gitignored, read by the framework)
```

Decrypt once at startup. The app never calls age again: it reads the plaintext file like any other config. Only the `.age` ciphertext is committed; the plaintext is gitignored and must never be committed.

## Plain age commands

```bash
# Encrypt for every recipient (one -r per public key), then commit the .age file
age -r age1... -r age1... -o .secrets.toml.age .secrets.toml

# Decrypt with this machine's key; the plaintext stays gitignored
age -d -i ~/.config/age/age.txt -o .secrets.toml .secrets.toml.age
```

Take the recipient list from [Age keys and recipients](keys_And_Recipients.md). Changing the list means decrypting and encrypting again: the old ciphertext still opens for the old recipients.

## Worked example: wrapper script in a Dynaconf repo

A repo with several secret files can wrap the two commands in one script so the recipients live in a single place. The Python poster repo (`02_Poster_Versions/01_Python_Poster`) does this: its secrets live in `Input/Dynaconf/Secrets/` and are managed by `Input/Dynaconf/manage_secrets.sh`. This section describes that script only; other repos may use the plain commands above.

```bash
# Decrypt to disk (safe to run repeatedly, overwrites plaintext)
./Input/Dynaconf/manage_secrets.sh decrypt

# Decrypt only what is missing (idempotent, use at startup and in scripts)
./Input/Dynaconf/manage_secrets.sh ensure

# Encrypt plaintext back to .age (after editing secrets)
./Input/Dynaconf/manage_secrets.sh encrypt
```

`ensure` is the safe default for CI and startup scripts: it skips files that are already decrypted and fails loudly if a `.age` file is missing entirely.

The script reads the identity from `$AGE_IDENTITY`, falling back to `~/.config/age/age.txt`. Its recipients are a `RECIPIENTS` array inside the script.

```bash
# Check which identity is active
echo ${AGE_IDENTITY:-~/.config/age/age.txt}

# Override for this session
export AGE_IDENTITY=~/.config/age/age.txt
```

**Editing secrets**

1. Decrypt: `./Input/Dynaconf/manage_secrets.sh decrypt`
2. Edit the plaintext `.toml` file in `Input/Dynaconf/Secrets/`
3. Re-encrypt: `./Input/Dynaconf/manage_secrets.sh encrypt`
4. Commit the `.age` file.

**Adding a recipient**

1. Get the new machine's public key (see [Age keys and recipients](keys_And_Recipients.md)).
2. Add it to the `RECIPIENTS` array in `manage_secrets.sh`.
3. Decrypt, then encrypt, so the new key is included.
4. Commit `manage_secrets.sh` and the updated `.age` files.

**Files managed**

| Ciphertext (committed) | Plaintext (gitignored) | Purpose |
|---|---|---|
| `.secrets.toml.age` | `.secrets.toml` | Platform API tokens, credentials |
| `.ai_Secrets.toml.age` | `.ai_Secrets.toml` | AI API keys (OpenAI, Anthropic, etc.) |
