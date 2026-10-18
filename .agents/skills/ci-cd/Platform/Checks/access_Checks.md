# Access Checks

Use this after platform checks pass and before repo trigger checks. This skill assumes access already works; if an access check fails, use the runtime CI/CD repo docs to fix it.

## 1. Regular Access Check

From the project repo, confirm the remote points to Forgejo:

```bash
git remote -v
```

Then check whether the current machine/user can read the remote:

```bash
git ls-remote origin HEAD
```

Expected: a commit hash and `HEAD`. If this fails, stop here and use the runtime CI/CD repo docs.

Check Woodpecker dashboard reachability:

```bash
curl -sI https://ci.zetunweb.com | head -10
```

Expected: a Woodpecker response or OAuth redirect. This checks reachability, not browser login.

If browser/UI access is already available, quickly confirm:

- the target repo is visible in Woodpecker,
- the repo can be enabled or is already enabled,
- the Forgejo repo settings/webhook page is visible.

If these pass, continue to [application trigger checks](../../Application/Trigger/repo_Trigger_Checks.md).

## 2. If Access Is Missing

If the regular check fails, setup or access may be missing or stale. Do not rebuild it from this skill. Read the runtime CI/CD repo (`CI_CD_Home`, its folder is in `path_File.md` of the `web-comp-tools` skill): `Project_Manag/Docs/doc_Start.md` inside it.

For setup routing inside that repo, start with:

```text
Project_Manag/Docs/Setup/linker_Setup.md
```

Stop and ask the user when SSH keys, passwords, one-time login codes, OAuth approval, or decrypted secret material are needed.
