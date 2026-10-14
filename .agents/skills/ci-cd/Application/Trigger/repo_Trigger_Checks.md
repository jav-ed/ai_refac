# Repo Trigger Checks

Use this when platform checks and access checks pass, but a specific project does not run a Woodpecker pipeline. This file checks the application-side trigger path only. For API-backed repair after a missing piece is confirmed, use [repo trigger setup](repo_Trigger_Setup.md).

## Contents

- [Check local pipeline files](#1-check-the-pipeline-file-exists)
- [Check Forgejo state](#2-check-the-pipeline-file-is-on-forgejo)
- [Check activation and webhook APIs](#3-check-repo-activation-and-webhook-by-api)
- [Cross-check the UI](#4-optional-ui-cross-check)
- [Trigger and observe](#5-trigger-and-observe)

## 1. Check The Pipeline File Exists

From the project repo:

```bash
test -f .woodpecker.yml
test -f justfile
test -d Code/CI_CD
git ls-files .woodpecker.yml justfile Code/CI_CD
git status --short .woodpecker.yml justfile Code/CI_CD
```

Expected:

- `.woodpecker.yml` exists.
- `.woodpecker.yml`, `justfile`, and `Code/CI_CD/` are tracked.
- no uncommitted CI changes are required for the expected run.

Compare the local file shape with the skill template:

```text
.agents/skills/ci-cd/Template/.woodpecker.yml
.agents/skills/ci-cd/Template/justfile
.agents/skills/ci-cd/Template/Code/CI_CD/
```

Active baseline:

```yaml
skip_clone: true

variables:
  - &base_step
    image: bash
    environment:
      CI_REPO_PATH: /absolute/path/to/project/repo

when:
  - event: push
    branch: [main]
```

Use `test` as well only if the project intentionally uses the optional `test` branch:

```yaml
when:
  - event: push
    branch: [main, test]
```

Woodpecker should only use `CI_REPO_PATH` to call `Code/CI_CD/Steps/*.sh` scripts through absolute paths. Just remains available for local shortcut recipes, but CI should not depend on just.

Check that the path is intentional and points at a repo root:

```bash
rg -n "CI_REPO_PATH" .woodpecker.yml
test -f "$(rg -o 'CI_REPO_PATH:\\s*.*' .woodpecker.yml | sed 's/.*CI_REPO_PATH:\\s*//')/Code/CI_CD/Steps/01_Install.sh"
```

## 2. Check The Pipeline File Is On Forgejo

Woodpecker reads `.woodpecker.yml` from Forgejo, not from uncommitted local files.

```bash
git branch --show-current
git ls-remote origin HEAD
git show origin/main:.woodpecker.yml >/dev/null
git log --oneline origin/main..main
```

Expected:

- `origin/main` exists.
- `origin/main` contains `.woodpecker.yml`.
- `origin/main..main` is empty, or any listed commits are intentionally not pushed yet.

If `.woodpecker.yml` exists locally but not on `origin/main`, push or stop here. Woodpecker may receive the webhook but filter the pipeline because the config is missing from the triggering branch.

## 3. Check Repo Activation And Webhook By API

Run from the skill secret folder:

```bash
cd .agents/skills/ci-cd/Platform/Secrets
```

First verify that both API tokens are accepted:

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

If either preflight returns 401 or 403, stop. The token is stale, missing required scope, or not accepted by the live service. Refresh access before continuing with API-backed checks or setup.

Check Forgejo repo existence and permissions:

```bash
fnox exec -- sh -c 'curl -fsS \
  -H "Authorization: token $FORGEJO_API_TOKEN" \
  -H "Accept: application/json" \
  "https://git.zetunweb.com/api/v1/repos/OWNER/REPO"'
```

Expected:

- HTTP 200.
- `full_name` is the intended repo.
- `permissions.admin` is `true` for API activation work.

Check Forgejo hooks:

```bash
fnox exec -- sh -c 'curl -fsS \
  -H "Authorization: token $FORGEJO_API_TOKEN" \
  -H "Accept: application/json" \
  "https://git.zetunweb.com/api/v1/repos/OWNER/REPO/hooks"'
```

Expected when active:

- at least one hook,
- `type` is `forgejo`,
- `active` is `true`,
- `events` includes `push`,
- `config.content_type` is `json`.

Check Woodpecker repo state:

```bash
fnox exec -- sh -c 'curl -fsS \
  -H "Authorization: Bearer $WOODPECKER_API_TOKEN" \
  -H "Accept: application/json" \
  "https://ci.zetunweb.com/api/repos/lookup/OWNER/REPO"'
```

Expected when active:

- HTTP 200.
- `full_name` is the intended repo.
- `active` is `true`.

Missing-state meanings:

- Forgejo repo returns 404: the repo path is wrong or the repo does not exist.
- Forgejo hooks return `[]`: no webhook is configured.
- Woodpecker lookup returns 404: Woodpecker does not know or has not enabled the repo.

If either Forgejo hook state or Woodpecker repo state is missing, go to [repo trigger setup](repo_Trigger_Setup.md).

## 4. Optional UI Cross-Check

The API check is the primary check. UI can still help with delivery history:

- Woodpecker UI: repo is visible, enabled, and points at the expected Forgejo project.
- Forgejo repo settings: Woodpecker webhook exists and recent deliveries appear after a push.

If UI access is missing, go back to [access checks](../../Platform/Checks/access_Checks.md). If setup details are needed, use the runtime CI/CD repo docs.

## 5. Trigger And Observe

When the project is ready for the required production trigger check:

```bash
git push origin main
```

Watch Woodpecker server logs while pushing:

```bash
journalctl --user -u woodpecker-server.service -f
```

Expected:

- Forgejo sends the webhook.
- Woodpecker receives the event.
- a pipeline appears in the Woodpecker UI.

If no pipeline appears:

- no Woodpecker log entry usually means the webhook did not reach Woodpecker,
- webhook delivered but no pipeline usually means `.woodpecker.yml` is missing on the pushed branch, branch filters do not match, or the repo is disabled/untrusted.

Do not solve Forgejo/Woodpecker setup from this file. Use the runtime CI/CD repo docs for deeper webhook, OAuth, service, or permission fixes.
