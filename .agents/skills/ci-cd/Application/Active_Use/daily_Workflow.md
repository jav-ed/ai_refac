# Daily Workflow

Active use once the pipeline is integrated. For first-time setup, see [first_Setup.md](../Integration/first_Setup.md).

## Branch Workflow

| Branch | Purpose | Pipeline fires? | Where it deploys |
|--------|---------|-----------------|------------------|
| `pre` | Day-to-day dev | No | - |
| `main` | Production | Yes | `<deployment-root>/<domain>/main/dist` |

`main` is mandatory for deployed projects. The optional `test` branch belongs to the WIP internal non-production environment design; see [Optional internal test environment](../../Deferred/test_Environment.md).

Minimum branch filter:

```yaml
when:
  - event: push
    branch: [main]
```

## How A Push Triggers The Pipeline

1. Push to Forgejo or another Woodpecker-supported forge.
2. The forge webhook sends a push event to Woodpecker.
3. Woodpecker reads `.woodpecker.yml`.
4. The local agent runs `$CI_REPO_PATH/Code/CI_CD/Steps/*.sh` on the host.
5. Deploy copies `dist/` into a fresh release folder and swaps the lane `dist` symlink ([release layout](../../Architecture/release_Layout.md)).

## Manual Runs

Run from the project root:

```bash
just Step_01_Install
just Step_02_Build
just Step_03_Og_Images
just Step_04_Compress
```

`just` is only a local shortcut wrapper around the scripts. The matching scripts can still be called directly.

Run the deploy only when the project is approved for production deployment:

```bash
just Step_05_Deploy
```

## Rollback

Releases and the live `dist` symlink are described in [Release layout, retention and rollback](../../Architecture/release_Layout.md), which also says when a deploy rolls back by itself. To go back by hand, list the releases:

```bash
ssh <deploy-host> "ls -lt <deployment-root>/<domain>/main/releases/"
```

Swap to a previous release:

```bash
ssh <deploy-host> "ln -sfn releases/<release-id> <deployment-root>/<domain>/main/dist"
```

## Verify A Release

```bash
ssh <deploy-host> "readlink <deployment-root>/<domain>/main/dist"
ssh <deploy-host> "curl -sI https://<domain> | head -10"
ssh <deploy-host> 'curl -sI -H "Accept-Encoding: br" https://<domain> | grep -i content-encoding'
```

## Debug

```bash
journalctl --user -u woodpecker-server.service -f
journalctl --user -u woodpecker-agent.service -f
ssh <deploy-host> 'sudo journalctl -u caddy --since "5 min ago" --no-pager'
```

## Common Failure Modes

- **Pipeline runs but deploy fails with SSH.** Test `ssh <deploy-host> whoami` as the Woodpecker agent user.
- **Build passes locally but fails in CI.** Check uncommitted files and stale `node_modules/` because clone is skipped.
- **OG generation fails.** Rebuild or inspect the external OG generator, then rerun `Code/CI_CD/Steps/03_Og_Images.sh`.
- **Compression fails.** Check `CI_COMPRESSOR_BIN` in `Code/CI_CD/Config/env.env` and confirm `dist/` exists.
- **Deploy fails because the expected local output is missing.** Rerun `Code/CI_CD/Steps/02_Build.sh`, then OG generation and compression as needed. The generic template must not assume `dist/index.html`; a project may validate its own known entrypoint when that route is part of its explicit contract.
- **Deploy succeeds but the site does not update.** Check `readlink <deployment-root>/<domain>/main/dist`.
- **Need a non-production inspection target.** Do not improvise `test.<client-domain>`. Use the deferred internal environment notes first.
