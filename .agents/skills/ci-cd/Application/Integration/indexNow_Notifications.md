# IndexNow Notifications

Use this opt-in path for a retained-release Astro site that should notify IndexNow only when its rendered, search-relevant pages genuinely change. The shared Rust `indexnow` binary owns parsing, semantic fingerprints, comparison, key verification, HTTP submission, and report generation. The website owns this adapter and its explicit site values.

Do not add IndexNow automatically to every static site. What the binary does, its subcommands, and what a site needs before it qualifies (an active release symlink, a retained previous release, an HTTPS origin, a sitemap, a reviewed ownership key) are in the `web-comp-tools` skill, on its Index_Now page. This page covers the adapter: the overlay, the variables, the bootstrap state, the deploy call, and recovery.

Stay in this guide for routine integration and operation. Read the [IndexNow adapter contract](Detail/indexNow_Architecture.md) only when changing deployment ordering, bootstrap state, locking, or where reports are stored. Changing sitemap consumption, comparison behavior, or the report fields happens in the binary itself: the `web-comp-tools` skill names its docs.

## Contents

- [Reusable overlay](#reusable-overlay)
- [Project-owned files](#project-owned-files)
- [Explicit variables](#explicit-variables)
- [Bootstrap state](#bootstrap-state)
- [Deploy integration](#deploy-integration)
- [Failure and recovery](#failure-and-recovery)
- [Verification](#verification)

## Reusable Overlay

The copyable adapter lives at `Template/IndexNow/`. It is an overlay rather than part of the default Astro template because IndexNow is opt-in.

```text
Template/IndexNow/
└── Code/CI_CD/
    ├── Lib/
    │   ├── indexnow.sh
    │   └── IndexNow/
    │       ├── config.sh
    │       ├── fixture.sh
    │       ├── release.sh
    │       ├── remote_Submission.sh
    │       ├── state.sh
    │       └── submission.sh
    └── Tools/
        ├── indexnow_Bootstrap.sh
        └── indexnow_Dry_Run.sh
```

Copy the overlay into the website repository, preserving paths and executable permissions. Then integrate the project-owned values, deploy call, Just recipes, and public key described below. Do not replace the site's complete env file, deploy step, or justfile with generic snippets.

The thin `Code/CI_CD/Lib/indexnow.sh` entrypoint is stable. Deployment and operator scripts source only that file; the `IndexNow/` subfolder owns the separate configuration, release, state, fixture, local orchestration, and remote-execution responsibilities.

## Project-Owned Files

```text
Public/<key>.txt                       # or the configured Astro publicDir
Code/CI_CD/Config/env.env             # explicit site values
Code/CI_CD/Lib/indexnow.sh             # reusable adapter entrypoint
Code/CI_CD/Lib/IndexNow/               # reusable implementation
Code/CI_CD/Tools/indexnow_Dry_Run.sh   # network-free candidate inspection
Code/CI_CD/Tools/indexnow_Bootstrap.sh # one-time recovery command
Code/CI_CD/Steps/05_Deploy.sh          # activation then notification
justfile                               # human-facing commands
```

Run production comparison on the deployment host because both retained releases already exist there. The local dry run copies only sitemap XML and HTML from the active release when the site is already initialized.

## Explicit Variables

Add reviewed values to `Code/CI_CD/Config/env.env`:

```bash
CI_INDEXNOW_LOCAL_BIN="/absolute/local/path/to/indexnow"
CI_INDEXNOW_REMOTE_BIN="/absolute/remote/path/to/indexnow"
CI_INDEXNOW_SITE_URL="https://example.com/"
CI_INDEXNOW_SITEMAP_PATH="sitemap-index.xml"
CI_INDEXNOW_KEY_FILE_NAME="<public-key>.txt"
CI_INDEXNOW_ENDPOINT="https://api.indexnow.org/indexnow"
CI_INDEXNOW_TIMEOUT_SECONDS="15"
CI_INDEXNOW_REPORT_DIR_NAME="indexnow-reports"
```

Do not invent defaults. A copied adapter must fail until every binary path, origin, sitemap, ownership filename, endpoint, timeout, and report location is explicit.

`CI_INDEXNOW_LOCAL_BIN` is the binary the dry run uses on the developer machine. Point it at a symlink in a folder that is on `PATH` (for example `~/.local/bin/indexnow`, linked to the tool repository's `target/release/indexnow`), written as an absolute path because the scripts test it with `-x`. A rebuild then updates the link's target in place, and a moved tool repository needs one new link instead of an edit in every site.

One optional switch: `CI_INDEXNOW_EXTENSIONLESS_URLS="true"` for a site whose sitemap lists page URLs without a trailing slash or file extension (`/docs/usage`, as Fumadocs links them) on a host that serves them as `<path>/index.html`. Unset or `false` keeps the binary strict (only `/` and `.html` URLs), any other value is a hard error. The binary must be new enough to know `--extensionless-page-urls`; an older one stops with an unknown-argument error.

The overlay assumes the Astro build folder `dist`. A site with another build root (Fumadocs: `dist/client`) changes the `dist` references in `Tools/indexnow_Dry_Run.sh` and passes its own root to `ci_indexnow_verify_local_key` in the deploy step.

The ownership value is public, not secret. The root URL proves submission authority for that host. File content must exactly match the filename without `.txt`, with at most one final newline.

## Bootstrap State

`<lane>/<report-dir>/bootstrap.json` is the only initialization evidence. A missing report selects full-site bootstrap; a valid report selects incremental comparison. A malformed report, incomplete `.tmp` report, or stale lock is a hard error.

Never infer initialization from an incremental report, a deployment date, or a zero-diff comparison. Those facts cannot prove that search engines received the unchanged URLs.

Store the report on the deployment host beside the lane's retained releases:

```text
<deployment-root>/<domain>/<lane>/<report-dir>/bootstrap.json
```

Do not store it in Git or inside the Caddy-served `dist/` tree. See the architecture detail for the complete evidence and storage contract.

## Deploy Integration

Integrate the adapter into `Code/CI_CD/Steps/05_Deploy.sh`:

1. Source `Code/CI_CD/Lib/indexnow.sh`.
2. Keep the site's existing validation, upload, and atomic activation flow.
3. Call `ci_indexnow_notify_remote` with the exact previous target, current target, and release name after activation.
4. Apply retention only after notification and report finalization succeed (retention rule: [Release layout, retention and rollback](../../Architecture/release_Layout.md)).

Do not change build steps 01 through 04 for IndexNow. The deployment state machine and exact-release invariants live in the architecture detail.

## Failure and Recovery

IndexNow failure does not roll back an already activated static release, but it must fail CI and skip retention so required roots remain available.

Expose both operator commands:

```make
# Compare production state with local dist without network submission.
IndexNow_Dry_Run:
    Code/CI_CD/Tools/indexnow_Dry_Run.sh

# Recover the one-time full-site submission after its release is already active.
IndexNow_Bootstrap:
    Code/CI_CD/Tools/indexnow_Bootstrap.sh
```

`IndexNow_Bootstrap` is a one-shot recovery path. It refuses before contacting IndexNow when a valid bootstrap report already exists. Automatic deployment and manual recovery share the same remote lock, so they cannot submit concurrently.

A stale lock or malformed report requires explicit diagnosis. Do not delete either automatically because it may represent interrupted external state.

## Verification

Before activation, prove:

```bash
find Code/CI_CD/Lib/IndexNow Code/CI_CD/Tools -type f -name '*.sh' -exec bash -n {} +
bash -n Code/CI_CD/Lib/indexnow.sh
bash -n Code/CI_CD/Steps/05_Deploy.sh
just --list
just Step_01_Install
just Step_02_Build
just Step_03_Og_Images
just Step_04_Compress
just IndexNow_Dry_Run
```

Also verify that the configured Astro `publicDir` copied the ownership file into the root of `dist/`. Conventional `public/` is wrong when Astro uses a custom public directory.

The first live proof must use the normal reviewed production deployment. Inspect its log, `bootstrap.json`, the public ownership URL, and the public sitemap. An accepted IndexNow response confirms receipt, not crawling or indexing.
