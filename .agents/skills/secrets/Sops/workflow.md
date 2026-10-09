# Sops: Field-Level File Encryption (Default)

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

Sops is installed as a shared mise system tool (tool name `sops`) and linked at `/usr/local/bin/sops`; the `default-tools` skill describes the install procedure.

Verify:

```bash
sops --version
```

## Setup (per machine)

Sops reads the machine's age key at `~/.config/sops/age/keys.txt`, a symlink to the canonical `~/.config/age/age.txt`. Create the key and the symlinks once as described in [Age keys and recipients](../Age/keys_And_Recipients.md).

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
      age1...,age1...
  - path_regex: ^Config/.+\.yaml$
    age: >-
      age1...,age1...
```

Replace the `age1...` placeholders with the comma-separated recipient list from [Age keys and recipients](../Age/keys_And_Recipients.md).

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

Standard sops's `stores/` packages cover YAML, JSON, INI, ENV, BINARY only — no TOML store as of v3.12.2 (latest). For Dynaconf and other `.toml` files, use [raw age](../Age/workflow.md). For new work that needs field-level encryption, prefer YAML or JSON.

If you must put TOML through sops, the only supported route is `--input-type binary`, which encrypts the whole file as one blob. That removes the field-level visibility benefit — at which point raw age is simpler and more honest.

## Known age recipients

The public keys of our devices, the canonical key location and how to add a device are in [Age keys and recipients](../Age/keys_And_Recipients.md). After changing recipients in `.sops.yaml`, run `sops updatekeys` on the affected files (see Key rotation above).
