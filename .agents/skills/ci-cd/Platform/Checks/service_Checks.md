# Platform Checks

Verify the base CI platform before debugging project-specific pipeline files. This skill assumes the platform already works; if a platform check fails, use the runtime CI/CD repo docs to fix it.

## Runtime Context Repo

The current org runtime repo for Woodpecker is:

```text
/home/jav/Schreibtisch/Javed/0_Right_Sirat/1_Code/01_Projects/1_Web_Service/02_Workfl_Comp/CI_CD
```

It owns local service docs, runtime config, encrypted secrets, ignored upstream reference clones under `Repos/`, and broader source/debugging context. Go there for setup or repair details that are larger than a project pipeline check.

## Check Forgejo Reachability

From the machine running Woodpecker:

```bash
curl -sI https://git.zetunweb.com | head -10
```

Expected: HTTP headers from Forgejo or the fronting reverse proxy. If this fails, stop here and use the runtime CI/CD repo docs.

## Check Woodpecker Server And Agent

```bash
systemctl --user status woodpecker-server.service --no-pager
systemctl --user status woodpecker-agent.service --no-pager
```

Expected: both services active. If either is failed, gather the recent logs, then use the runtime CI/CD repo docs for the fix:

```bash
journalctl --user -u woodpecker-server.service -n 120 --no-pager
journalctl --user -u woodpecker-agent.service -n 120 --no-pager
```

## Check Dashboard Reachability

```bash
curl -sI https://ci.zetunweb.com | head -10
```

Expected: Woodpecker dashboard response or redirect to Forgejo OAuth. If this fails, stop here and use the runtime CI/CD repo docs.

For local backend diagnostics on `javPc`, the Woodpecker HTTP server still listens on `127.0.0.1:8000` behind Caddy.

## Check Local Backend Readiness

The agent must be running with the local backend and have host tools available through its service environment. If local-backend or PATH behavior looks wrong, stop here and use the runtime CI/CD repo docs.

## Check Upstream Docs

When exact Woodpecker behavior matters, use the local upstream docs clone in the CI/CD runtime repo:

```text
Repos/woodpecker/docs
```

The most relevant files are listed in that repo's `Project_Manag/Docs/Setup/repos_List.md`.
