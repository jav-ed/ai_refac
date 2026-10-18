# Release layout, retention and rollback

How a static site lands on the host and how an older version is kept or put back. This page is the one place that says it; the deploy template, the architecture page, the daily workflow and the stack skills point here instead of describing it again. It states the general rule. A repository may override it where it has a reason, and then says so in its own docs (see the end of this page).

## Layout on the host

Every deploy writes into its own release folder. The folder Caddy serves is a symlink that points at one of them:

```text
<deployment-root>/<domain>/
|-- domain.caddy
`-- main/
    |-- dist -> releases/<release-id>
    `-- releases/
        |-- 2026-06-16_p12_abc12345/
        `-- 2026-06-15_p11_9f3e21aa/
```

A release id is `<date>_p<pipeline-number>_<commit-prefix>`, for example `2026-06-16_p12_abc12345`. `main` is the one mandatory lane. `dist` is never the folder rsync writes into.

## Activation

The deploy step:

1. refuses when the exact release folder already exists,
2. refuses when `main/dist` exists and is not a symlink,
3. creates `main/releases/<release-id>/`,
4. rsyncs the built site into that folder,
5. points `main/dist` at it with one `ln -sfn`, so the switch is atomic and Caddy needs no reload,
6. removes old release folders beyond the retention count.

The generic template does not demand `dist/index.html`: a valid site may start at `dist/de/index.html`. A repository may check its own known entrypoint, but that check is not copied back into the template as a universal rule.

## Retention

- The newest `CI_DEPLOY_KEEP_RELEASES` folders stay, ordered by modification time. Zetun_Web and Refac_Docs use `5`.
- The value lives in `Code/CI_CD/Config/env.env` and is required: a deploy without it stops instead of guessing.
- Use `2` or more. The previous release is the rollback target. `3` means the current release and two backups.
- Pruning is the last thing a deploy does. It runs after the swap and after every check that can still roll back. When IndexNow notification is enabled it runs only after the notification succeeded, so a failed notification leaves both releases on the host for diagnosis.

## Rollback

Manually, re-point the symlink at the release you want and list the choices first:

```bash
ssh <deploy-host> "ls -lt <deployment-root>/<domain>/main/releases/"
ssh <deploy-host> "ln -sfn releases/<release-id> <deployment-root>/<domain>/main/dist"
```

Automatically, a deploy step that has live checks re-points the symlink at the previous release when a check fails, and removes the link again on a first release that has no previous one. The generic template has no live check. The Fumadocs pipeline adds a health URL (`CI_HEALTH_URL`) and one live social card (`CI_CARD_URL`, `CI_CARD_TYPE`), both with this rollback. A release that is wrong but still answers `200` is not caught by it; the check step before the deploy is the guard for that. A failed IndexNow notification does not roll back: the release stays active, the pipeline fails, and pruning is skipped.

## Overriding it in a repository

The numbers and the order above are the general rule. A repository may change:

- the retention count, in its own `env.env`,
- the checks of its deploy step: an entrypoint check, a health URL, a live-asset check, with rollback,
- anything else it needs, in its own copy of the step.

It then says what differs and why in its own docs, and the skill's template stays generic. A project-specific rule that turns out to help every site is moved here and into the template, not left in one repository.
