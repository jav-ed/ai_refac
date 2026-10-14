# Secrets And Access

Use this when a CI/CD platform check or runtime service needs access to Woodpecker, Forgejo, or Caddy secrets.

## Secret File Boundary

The `ci-cd` skill stores the complete secret set here:

```text
.agents/skills/ci-cd/Platform/Secrets/fnox.toml
```

This file is part of the skill on purpose. If the skill folder is copied to another repo or machine, the secret variable map and encrypted values needed by the workflow move with it.

In the CI_CD repo, the root `fnox.toml` is only a convenience symlink to the skill-owned file:

```text
fnox.toml -> .agents/skills/ci-cd/Platform/Secrets/fnox.toml
```

That symlink lets systemd services and root-run commands keep using `fnox exec` from the repo root, while preserving one real encrypted file inside the portable skill.

Do not assume API tokens still have the required permissions; verify with a harmless read endpoint before mutating anything.

## How To Use Fnox

Use the Fnox part of the `secrets` skill before running secret-backed commands.

Run from the skill secret folder or, in the CI_CD repo, from the repo root symlink. Both should resolve to the same encrypted file. From another repository root, pass the file explicitly with `fnox exec -c .agents/skills/ci-cd/Platform/Secrets/fnox.toml -- ...`.

### Passing a secret to a command

Writing `fnox exec -- curl -H "Authorization: Bearer $WOODPECKER_API_TOKEN" ...` does not work: your own shell expands `$WOODPECKER_API_TOKEN` before Fnox injects anything, so curl sends an empty value and the service answers 401. Verified against Woodpecker on 2026-10-03. Start the command that reads the variable inside Fnox and keep the variable unexpanded with single quotes:

```bash
fnox exec -- sh -c 'curl -fsS -H "Authorization: Bearer $WOODPECKER_API_TOKEN" https://ci.zetunweb.com/api/user'
```

All curl examples in this skill use that form. A Python or other script that reads the variable from its environment needs no wrapper.

## Stop Before Interactive Or Secret-Gated Work

Stop and ask the user when:

- `fnox doctor` fails.
- the local age identity cannot decrypt `fnox.toml`.
- an OAuth login, access code, or interactive dashboard approval is required.
- a command would display decrypted secrets in terminal output.

## Documentation Boundary

The skill may document:

- expected secret names,
- which service consumes them,
- the skill-owned `fnox.toml` path,
- which default-tools Fnox doc to read before running commands.

The skill must not contain:

- decrypted values,
- access codes,
- one-time OAuth tokens.
