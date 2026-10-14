# Read A Pipeline Run's Output

Use this to see what Woodpecker actually did for a push: whether a run exists, which step failed, and the step's log. Read the run before guessing why a build, package or deploy did not happen.

## Access

Woodpecker is at `https://ci.zetunweb.com`. The token `WOODPECKER_API_TOKEN` lives in the skill-owned `Platform/Secrets/fnox.toml`. Read the Fnox part of the `secrets` skill and the [secrets boundary](../../Platform/Secrets/secrets_And_Access.md) first. The stored token belongs to an admin user, so use GET requests only, never print the token, and stop and ask the user on a 401 or 403.

## Commands

Run from the repository root with an explicit Fnox config:

```bash
CFG=.agents/skills/ci-cd/Platform/Secrets/fnox.toml
HELPER=.agents/skills/ci-cd/Application/Active_Use/pipeline_Output.py

fnox exec -c $CFG -- python3 $HELPER OWNER/REPO                      # recent pipelines, newest first
fnox exec -c $CFG -- python3 $HELPER OWNER/REPO 259                  # steps of pipeline 259
fnox exec -c $CFG -- python3 $HELPER OWNER/REPO 259 05_Deploy --tail 60   # last 60 log lines of one step
```

`OWNER/REPO` is the Forgejo name, for example `my-org/my-app`. The helper makes GET requests only.

## Reading the result

- A pipeline whose commit matches the push proves Woodpecker received it. No newer pipeline than the last known one means the push did not reach Forgejo, was filtered, or the webhook did not fire: continue with [trigger checks](../Trigger/repo_Trigger_Checks.md).
- Steps run in order. The first `failure` is the cause; later steps show `skipped`.
- A step can fail after earlier steps succeeded and still leave the old release serving. Read the log tail for the reason before deciding what to do.

## Notes

- The log API returns one base64 entry per line, without newlines, and some entries have no data. The helper joins them with newlines and skips the empty ones.
- Do not copy tokens, URLs with credentials, or decrypted values from a log into reports.
