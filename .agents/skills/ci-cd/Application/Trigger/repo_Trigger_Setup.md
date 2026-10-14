# Repo Trigger Setup

API-backed handoff for making a Forgejo repo actually trigger Woodpecker after read-only checks prove trigger state is missing.

Use [repo trigger checks](repo_Trigger_Checks.md) first. Do not activate a repo just because files exist locally; activation creates Woodpecker repo state and Forgejo webhook state, so future pushes to matching `.woodpecker.yml` branches can trigger CI.

## Safety Boundary

Use this only after [repo trigger checks](repo_Trigger_Checks.md) show that Woodpecker repo state or Forgejo webhook state is missing.

This is a mutating workflow. It creates Woodpecker repo state and asks Woodpecker to create the Forgejo webhook. Future pushes to branches accepted by `.woodpecker.yml` can then trigger CI.

## Get Forgejo Repo ID

Run from the skill secret folder:

```bash
cd .agents/skills/ci-cd/Platform/Secrets
```

Before any mutating request, verify token access with read-only calls:

```bash
fnox exec -- sh -c 'curl -fsS \
  -H "Authorization: token $FORGEJO_API_TOKEN" \
  -H "Accept: application/json" \
  "https://git.zetunweb.com/api/v1/user"'

fnox exec -- sh -c 'curl -fsS \
  -H "Authorization: Bearer $WOODPECKER_API_TOKEN" \
  -H "Accept: application/json" \
  "https://ci.zetunweb.com/api/user"'
```

If either token check fails with 401 or 403, stop. Do not attempt activation with stale or under-scoped tokens.

```bash
fnox exec -- sh -c 'curl -fsS \
  -H "Authorization: token $FORGEJO_API_TOKEN" \
  -H "Accept: application/json" \
  "https://git.zetunweb.com/api/v1/repos/OWNER/REPO"'
```

Use the returned `id` as Woodpecker's `forge_remote_id`.

## Enable Repo In Woodpecker

```bash
fnox exec -- sh -c 'curl -fsS -X POST \
  -H "Authorization: Bearer $WOODPECKER_API_TOKEN" \
  -H "Accept: application/json" \
  "https://ci.zetunweb.com/api/repos?forge_remote_id=FORGEJO_REPO_ID"'
```

Expected:

- HTTP 200.
- `full_name` matches `OWNER/REPO`.
- `active` is `true`.

Woodpecker's `POST /api/repos?forge_remote_id=...` activates the repo in Woodpecker and creates the Forgejo webhook. Do not create a separate manual Forgejo hook first unless this endpoint fails and you are deliberately repairing by hand.

## Verify Both Sides

Woodpecker:

```bash
fnox exec -- sh -c 'curl -fsS \
  -H "Authorization: Bearer $WOODPECKER_API_TOKEN" \
  -H "Accept: application/json" \
  "https://ci.zetunweb.com/api/repos/lookup/OWNER/REPO"'
```

Forgejo:

```bash
fnox exec -- sh -c 'curl -fsS \
  -H "Authorization: token $FORGEJO_API_TOKEN" \
  -H "Accept: application/json" \
  "https://git.zetunweb.com/api/v1/repos/OWNER/REPO/hooks"'
```

Expected:

- Woodpecker lookup returns HTTP 200 and `active: true`.
- Forgejo hooks include an active `forgejo` hook with `push` events and `json` content type.

## Tested Result For This Repo

For `zetun_Web/zetun_Web`, Forgejo returned repo ID `25`. Activating with:

```bash
POST https://ci.zetunweb.com/api/repos?forge_remote_id=25
```

returned HTTP 200 and created active Woodpecker repo `zetun_Web/zetun_Web` plus an active Forgejo webhook.

The trigger path is active, but first production-triggering pushes still require normal project safety checks: `.woodpecker.yml` must be present on `origin/main`, branch filters must match, and deploy values must be safe.
