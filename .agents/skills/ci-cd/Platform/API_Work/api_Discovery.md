# API Discovery

Use this when you need to know which API surface or feature hints are available before writing Forgejo/Woodpecker automation.

When using tokens from Fnox, start with read-only token checks. Do not run mutating API calls until the read-only checks succeed.

## Forgejo

Forgejo exposes API limits and version:

```bash
fnox exec -- sh -c 'curl -fsS \
  -H "Authorization: token $FORGEJO_API_TOKEN" \
  "https://git.zetunweb.com/api/v1/settings/api"'

fnox exec -- sh -c 'curl -fsS \
  -H "Authorization: token $FORGEJO_API_TOKEN" \
  "https://git.zetunweb.com/api/v1/version"'
```

Token preflight:

```bash
fnox exec -- sh -c 'curl -fsS \
  -H "Authorization: token $FORGEJO_API_TOKEN" \
  -H "Accept: application/json" \
  "https://git.zetunweb.com/api/v1/user"'
```

Expected: HTTP 200 and the authenticated user JSON. HTTP 401 or 403 means the stored token is stale, missing required scope, or not accepted by this Forgejo instance. Stop and refresh the token before using API-backed repair docs.

Forgejo also exposes Swagger:

```bash
curl -fsS "https://git.zetunweb.com/swagger.v1.json"
```

Useful signals:

- current Forgejo version,
- pagination and blob-size API limits,
- exact endpoint list and request/response schemas from Swagger.

## Woodpecker

Woodpecker exposes version and configured forges:

```bash
curl -fsS "https://ci.zetunweb.com/version"

fnox exec -- sh -c 'curl -fsS \
  -H "Authorization: Bearer $WOODPECKER_API_TOKEN" \
  "https://ci.zetunweb.com/api/forges"'
```

Token preflight:

```bash
fnox exec -- sh -c 'curl -fsS \
  -H "Authorization: Bearer $WOODPECKER_API_TOKEN" \
  -H "Accept: application/json" \
  "https://ci.zetunweb.com/api/user"'
```

Expected: HTTP 200 and the authenticated Woodpecker user JSON. HTTP 401 means the stored token is stale, malformed, or no longer accepted. Stop and refresh the token before running repo activation or webhook repair commands.

Woodpecker web config exposes some UI/runtime feature flags:

```bash
curl -fsS "https://ci.zetunweb.com/web-config.js"
```

Woodpecker Swagger is available when enabled:

```bash
curl -fsS "https://ci.zetunweb.com/swagger/doc.json"
```

Useful signals:

- Woodpecker version,
- Swagger enabled/disabled,
- configured forge type and URL,
- user-registered agents enabled/disabled,
- log line limits and UI runtime settings,
- exact endpoint list and request/response schemas from Swagger.

## Boundary

These endpoints help with general availability and automation shape. They do not prove a specific application repo trigger works. For that, use [repo trigger checks](../../Application/Trigger/repo_Trigger_Checks.md).
