# Fnox

Fnox stores encrypted secrets in `fnox.toml`, committed to git. Encrypted with native age keys — anyone whose `age1...` public key is listed as a recipient can decrypt.

## Setup (per machine)

```bash
age-keygen -o ~/.config/fnox/age.txt
chmod 600 ~/.config/fnox/age.txt
```

Fnox auto-discovers the key at `~/.config/fnox/age.txt`. No env var needed. The `age1...` public key printed by `age-keygen` is what goes into `recipients`.

```bash
fnox doctor   # verify identity is wired up
```

## Core commands

```bash
fnox exec -- <command>              # inject all secrets as env vars into subprocess
fnox get KEY_NAME                   # read a single secret
fnox set KEY_NAME "value"           # set or update (always double-quote)
fnox list                           # list all secrets in fnox.toml
fnox reencrypt -p age               # re-encrypt all (required after adding/removing a recipient)
```

> Fnox walks up the directory tree — run from anywhere inside the repo.
>
> The shared g12 installation is verified with Fnox 1.33.1.

## Selecting a config file

Fnox defaults to discovering a file named `fnox.toml` by walking up the directory tree. Use `-c <path>` or `--config <path>` when the file has another name or when a repository contains multiple independent secret sets. The selected file can have any filename; pass the option on every command that should use it.

```bash
fnox list -c rishta_Worker_Mails.toml
fnox get -c rishta_Worker_Mails.toml MARIA_BUTT_EMAIL_PASSWORD
fnox set -c rishta_Worker_Mails.toml MARIA_BUTT_EMAIL_PASSWORD
fnox exec -c rishta_Worker_Mails.toml -- <command>
fnox reencrypt -c rishta_Worker_Mails.toml -p age
```

Prefer an explicit config path in services and scripts. This prevents Fnox from silently selecting a different parent-directory `fnox.toml` when a command runs from another working directory.

**Known gotcha:** passing `--description` alongside stdin silently drops the value — fnox writes the entry without encrypting anything. Verified 2026-05-08. Do not combine `--description` with stdin. Add context as a comment in the file instead (comments survive fnox writes — see below).

## This server (g12)

Use explicit execution:

```bash
fnox exec -- my-command
```

## Comments — mandatory, not optional

Every secret in `fnox.toml` must have a comment above it. Variable names alone are not enough — a new developer (or agent) reading the file must immediately understand what the secret is, what uses it, and any relevant notes.

**Comments survive fnox writes.** Fnox only rewrites the specific line it touches. Adding, updating, or removing a secret leaves all surrounding comments intact. Verified 2026-05-08.

When adding a new secret, use this sequence:

1. Run `fnox set KEY_NAME` and enter the secret through stdin or the hidden prompt.
2. Immediately edit `fnox.toml` and add the required comment above the new key.
3. Run `fnox list` to confirm the key exists without printing the secret value.

Do not pass sensitive values as shell arguments unless the user explicitly accepts that exposure. Prefer the hidden prompt or piped stdin.

```toml
[secrets]
# Email password for javed@javedab.com
# Used by: Himalaya CLI → ~/.config/himalaya/config.toml (backend.auth.cmd = "fnox get ...")
JAVED_JAVEDAB_EMAIL_PASS = { provider = "age", value = "..." }

# Cloudflare API token — DNS management for zetunweb.com and javedab.com
# Permissions needed: Zone:DNS:Edit on both zones
CF_API_TOKEN = { provider = "age", value = "..." }
```

A bare key with no comment is incomplete. Always add the comment in the same edit session as adding the secret.

## fnox.toml structure

```toml
[providers]
age = { type = "age", recipients = ["age1...", "age1..."] }

[secrets]
# What this secret is — what uses it — any permission or rotation notes
MY_SECRET = { provider = "age", value = "<encrypted blob>" }
```

See [installation.md](installation.md) for the known `age1...` public keys for both devices.

## Troubleshooting

| Error | Cause | Fix |
|---|---|---|
| `no identity matched any of the recipients` | Key mismatch | Confirm your `age1...` public key is in `recipients` |
| `failed to decrypt` | Key file missing | Check `~/.config/fnox/age.txt` exists with `chmod 600` |
| Secret not found | Wrong directory or config | Run inside the intended `fnox.toml` tree, or select the file explicitly with `-c <path>` |
| `fnox doctor` fails | Identity not found | Re-run `age-keygen -o ~/.config/fnox/age.txt`, add public key to recipients |

## References

- [Installation and key setup](installation.md) — age-keygen, known recipients, init, add/remove a recipient
- [Environments and overrides](environments.md) — profiles (dev/staging/prod), hierarchical config, `fnox.local.toml`
- [Leases](leases.md) — short-lived cloud credentials (AWS STS, GCP, Vault); skip unless using cloud providers
