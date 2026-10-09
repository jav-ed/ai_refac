---
name: secrets
description: Manages encrypted secrets committed to git with sops (default, field-level encryption of YAML, JSON, INI and ENV files), raw age (whole-file encryption, used for TOML) and fnox (secrets injected as environment variables). Use when reading or setting an API key, token or password, running a process with secrets as env vars, editing or re-encrypting `.sops.yaml`, `.age` or `fnox.toml` files, or adding a device as a recipient.
---

# secrets

Credentials live encrypted in git and are decrypted on the machines whose age key is a recipient. Three tools do the work, all with age keys underneath. They differ in how the secret reaches the application: inside a config file the app already reads (sops, raw age), or as an environment variable injected before the process starts (fnox). Pick by what the consuming code wants.

## Pick the tool

| | **sops** (default for new work) | **raw age** | **fnox** |
|---|---|---|---|
| How code consumes the secret | Normal field in the parsed config | Normal field, after whole-file decrypt | `os.Getenv("KEY")` |
| File formats | YAML, JSON, INI, ENV | Any (opaque blob); used for TOML | n/a: values live in `fnox.toml` |
| Granularity | Per field within a file | Whole file | Per key |
| Edit flow | `sops edit file.yaml` | Decrypt, edit plaintext, re-encrypt | `fnox set KEY` |
| Best fit | Structured app or per-tenant config with a few secret fields | TOML or Dynaconf files, one-off whole-file encryption | Daemons, CLIs and scripts that want env vars |

Standard sops covers YAML, JSON, INI, ENV and BINARY only, not TOML. That is why TOML files use raw age.

## Common commands

```bash
sops edit config.yaml       # decrypt in $EDITOR, re-encrypt on save
sops decrypt config.yaml    # plaintext to stdout
fnox get KEY_NAME           # read one secret
fnox exec -- ./binary       # run with all secrets injected as env vars
fnox set KEY_NAME           # add or update through the hidden prompt, then add the comment
fnox list                   # confirm keys exist, never prints values
```

Fnox discovers `fnox.toml` by walking up the directory tree. Select another file with `-c <path>` on every command, and prefer an explicit path in scripts and services so a parent-directory file is never picked by accident.

## Rules for every tool

- Never print, log or echo secret values.
- Never pass a secret as a shell argument unless the user accepts that exposure; use the hidden prompt or stdin.
- Decrypted plaintext files are gitignored and never committed. Only ciphertext and public keys go to git.
- Every secret in `fnox.toml` needs a comment above it saying what it is and what uses it, added in the same edit session.
- After a recipient change, re-encrypt: `sops updatekeys`, `fnox reencrypt -p age`, or decrypt and encrypt again for raw age.

## Navigation

- [Sops workflow](Sops/workflow.md): installing sops, `.sops.yaml` creation rules, encrypting only selected fields, key rotation with `updatekeys`, and the TOML caveat. Open when creating or editing a sops-encrypted YAML, JSON, INI or ENV file.
- [Raw age workflow](Age/workflow.md): plain `age` encrypt and decrypt commands, the whole-file pattern for TOML and Dynaconf, and a worked example with a wrapper script. Open when a secrets file is TOML or a `.age` file is involved.
- [Age keys and recipients](Age/keys_And_Recipients.md): where each machine's private key lives, the public keys of our devices in the format each tool wants, and how to add or remove a device. Open when creating a new encrypted config or when a machine cannot decrypt.
- [Fnox](Fnox/linker_Fnox.md): setup, core commands, explicit config selection, the mandatory comment rule, `fnox.toml` structure and troubleshooting. It links installation, profiles for dev, staging and prod, and short-lived cloud credential leases.
