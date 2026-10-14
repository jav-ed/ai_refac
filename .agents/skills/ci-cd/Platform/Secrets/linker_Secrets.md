# Secrets

CI/CD secrets are handled through Fnox. Do not print decrypted values. Use this page to identify which secret names should exist and where the skill-owned `fnox.toml` lives.


## Required Secret Names

- `WOODPECKER_FORGEJO_CLIENT`: Forgejo OAuth client ID used by Woodpecker server.
- `WOODPECKER_FORGEJO_SECRET`: Forgejo OAuth client secret used by Woodpecker server.
- `WOODPECKER_AGENT_SECRET`: shared secret used by Woodpecker server and agent for registration/authentication.
- `WOODPECKER_GRPC_SECRET`: persistent JWT-signing secret used by the Woodpecker server for gRPC connections. Required from `3.17.0` onward so the key does not rotate across server restarts.
- `WOODPECKER_API_TOKEN`: Woodpecker API token used for repo activation, lookup, and trigger repair.
- `FORGEJO_API_TOKEN`: Forgejo API token used for repo and webhook checks.
- `CF_API_TOKEN`: Cloudflare API token used by Caddy for DNS-01 certificate issuance.


## File Boundary

The complete secret set lives in:

```text
.agents/skills/ci-cd/Platform/Secrets/fnox.toml
```

In the CI_CD repo, root `fnox.toml` is a symlink to that skill-owned file so runtime services can still run `fnox exec` from the repo root. Do not create a second real root secret file.


## Fnox Usage

Do not duplicate Fnox usage instructions here. Use the Fnox part of the `secrets` skill first.

Then run commands from `.agents/skills/ci-cd/Platform/Secrets/`, or from the CI_CD repo root when the symlink exists.
