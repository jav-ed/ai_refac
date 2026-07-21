# Sops — Field-Level File Encryption (Default)

Use sops when the application reads a structured config file (YAML, JSON, INI, ENV) and you want some fields encrypted in place. Sops decrypts on edit and re-encrypts on save; at runtime the application reads the file as it always would, after a one-shot decrypt or via library bindings.

For TOML files, use [raw age](../Age/workflow.md) instead — sops doesn't natively support TOML.

## Mental model

```
file.yaml in git           ←→  sops edit file.yaml        ←→  $EDITOR (plaintext)
   (sops: metadata,                                              ↓
    values are ENC[...])                                    save → sops re-encrypts → write back
```

Encrypted file stays in git. Plaintext only ever exists during an edit session or briefly at app startup.

## Install

Sops is managed via mise globally:

```bash
# Add to ~/.config/mise/config.toml under [tools]:
#   sops = "latest"
mise install sops
```

Verify:

```bash
sops --version   # expects 3.12.2 or later
```

## Setup (per machine)

The canonical age key lives at `~/.config/age/age.txt`. Both fnox and sops symlink to it — no duplicate key files, one source of truth.

```bash
mkdir -p ~/.config/sops/age
ln -sf ~/.config/age/age.txt ~/.config/sops/age/keys.txt
```

Or point sops at a different identity:

```bash
export SOPS_AGE_KEY_FILE=~/.config/somewhere/else.txt
```

## .sops.yaml — recipients declared once

At the repo root, create `.sops.yaml` so you never repeat recipients per file:

```yaml
creation_rules:
  - path_regex: ^Customers/.+\.yaml$
    age: >-
      age1uy9ps3p4460de20v8fgvt6gyg5ml0wscesd32m4693567aaumgnsspt8x5,age1areaucgxqcwvnsnr3mpz46g27pc9gf8qqcvaaa3k5fn3fluvde3s27cql9
  - path_regex: ^Config/.+\.yaml$
    age: >-
      age1uy9ps3p4460de20v8fgvt6gyg5ml0wscesd32m4693567aaumgnsspt8x5,age1areaucgxqcwvnsnr3mpz46g27pc9gf8qqcvaaa3k5fn3fluvde3s27cql9
```

When sops creates or edits a file, it walks up the tree to find `.sops.yaml`, matches the file path against `path_regex` (rules evaluated top-to-bottom, first match wins), and uses that rule's keys.

## Core commands

```bash
sops edit file.yaml           # decrypt in $EDITOR, re-encrypt on save
sops decrypt file.yaml        # plaintext to stdout
sops encrypt -i file.yaml     # encrypt an existing plaintext file in place
sops updatekeys file.yaml     # re-sync this file to the current .sops.yaml recipients
```

## Encrypting only some fields

By default sops encrypts every value. To restrict:

```bash
# Only encrypt values whose key matches the regex
sops encrypt --encrypted-regex '^(password|secret|token|key)$' -i file.yaml

# Or: leave specific keys plaintext, encrypt everything else
sops encrypt --unencrypted-regex '^(description|host|port)$' -i file.yaml
```

The same options can live in `.sops.yaml` per `creation_rule`. See upstream README "Encrypting only parts of a file".

## Key rotation

When recipients change in `.sops.yaml`:

```bash
sops updatekeys file.yaml                                  # one file
find . -name '*.enc.yaml' -exec sops updatekeys -y {} \;   # batch (-y skips confirm)
```

`updatekeys` rewrites only the `sops:` metadata block at the bottom of the file — the encrypted values themselves don't need re-encryption because sops uses a per-file data key.

## TOML caveat

Standard sops's `stores/` packages cover YAML, JSON, INI, ENV, BINARY only — no TOML store as of v3.12.2 (latest). For Dynaconf `.toml` files in this repo, use [raw age](../Age/workflow.md). For new work that needs field-level encryption, prefer YAML or JSON.

If you must put TOML through sops, the only supported route is `--input-type binary`, which encrypts the whole file as one blob. That removes the field-level visibility benefit — at which point raw age via `manage_secrets.sh` is simpler and more honest.

## Known age recipients

| Device | Public key |
|---|---|
| jav (local) | `age1uy9ps3p4460de20v8fgvt6gyg5ml0wscesd32m4693567aaumgnsspt8x5` |
| g12 (server) | `age1areaucgxqcwvnsnr3mpz46g27pc9gf8qqcvaaa3k5fn3fluvde3s27cql9` |

Native age keys, the same ones used by raw age and fnox. The canonical location on each device is `~/.config/age/age.txt`; `~/.config/fnox/age.txt` and `~/.config/sops/age/keys.txt` are symlinks to it. Add new devices by appending their `age1...` to the relevant `.sops.yaml` rules and running `sops updatekeys` on affected files.
