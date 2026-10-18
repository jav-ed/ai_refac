# IndexNow Adapter Contract

Use this developer detail only when changing how a site's deployment calls IndexNow
(ordering, bootstrap state, locking, report placement) or diagnosing an invariant
failure of that adapter. How the `indexnow` binary itself works (sitemap parsing,
fingerprints, comparison, submission, report fields) is documented with the binary;
the `web-comp-tools` skill names those docs. For installation, configuration, dry runs, recovery, and
normal deployment, use the [IndexNow operator guide](../indexNow_Notifications.md).

## Contents

- [Ownership boundaries](#ownership-boundaries)
- [Release selection](#release-selection)
- [Bootstrap and incremental state](#bootstrap-and-incremental-state)
- [Deployment transaction](#deployment-transaction)
- [Failure and concurrency](#failure-and-concurrency)
- [Report contract](#report-contract)
- [Maintenance checklist](#maintenance-checklist)

## Ownership Boundaries

The existing Astro pipeline keeps its current responsibilities:

- Astro generates the sitemap during the normal build.
- Steps 01 through 04 install, build, generate OG images, and compress without
  depending on IndexNow.
- Step 05 owns deployment and invokes IndexNow after activation because that is
  where both retained releases and the public ownership file are available.

IndexNow does not generate, rewrite, flatten, or repair the sitemap. Its adapter
only validates and consumes the completed build artifact.

The website repository owns:

- explicit site and deployment values;
- the public ownership-key file;
- the modular shell adapter and operator commands;
- retained-release selection and activation ordering;
- deployment-host report, lock, and recovery behavior.

The separate Rust `indexnow` binary owns everything about reading sitemaps, fingerprinting pages,
comparing releases, verifying ownership, submitting, and typed reports; see its own docs.

Woodpecker calls shell steps directly. Just exposes human-facing commands and is
not a CI dependency.

## Release Selection

Compare the exact release that was active before deployment with the exact release activated by
the current deployment. Never infer ancestry from the second-newest directory: manual activation
and rollback can change that order.

## Bootstrap And Incremental State

`<lane>/<report-dir>/bootstrap.json` is the only initialization evidence.

A valid bootstrap report proves:

- at least one URL was added from an empty sitemap fixture;
- changed, removed, and unchanged counts are zero;
- the public ownership key was verified;
- submitted count equals the complete added count;
- at least one submission batch was recorded.

Missing evidence selects bootstrap mode, even when the current site has no new
URLs relative to its previous release. This closes the historical gap for sites
that existed before IndexNow was integrated.

Valid evidence selects incremental mode and compares the exact previous and
current releases. A malformed report, incomplete temporary report, or stale
lock is a hard error rather than permission to guess or resubmit.

## Deployment Transaction

The required order is:

1. Validate deploy and IndexNow configuration.
2. Validate the completed local sitemap and ownership file.
3. Read and validate the current active release target.
4. Upload a new retained release.
5. Atomically activate the new release.
6. Determine bootstrap or incremental mode from validated remote state.
7. Verify the active target and acquire the shared report lock.
8. Compare releases, verify public ownership, and submit candidates.
9. Verify the active target again and atomically finalize the report.
10. Apply release retention only after IndexNow succeeds.

Submission happens after activation so the announced pages and key are publicly
resolvable. Retention happens last because comparison and failure diagnosis may
require both retained roots.

## Failure And Concurrency

Before activation, a failure leaves the active site unchanged. After activation,
an IndexNow failure:

- leaves the new site active;
- fails Woodpecker visibly;
- skips retention cleanup;
- preserves the comparison releases;
- removes temporary fixtures and reports when safe.

Automatic deployment and `IndexNow_Bootstrap` use the same atomic lock. They
must not contact IndexNow concurrently. A valid bootstrap report makes manual
recovery refuse before any network request.

Incremental reports use one final path per release identifier and are never
silently overwritten. An empty incremental candidate set is successful and
does not contact IndexNow.

## Report Contract

Reports live on the deployment host beside the lane's releases, outside the Caddy-served `dist/` tree
and outside Git, and reach their final path through an atomic rename. The fields and invariants of a
report are the binary's contract; bootstrap state, operator summaries, recovery, and diagnosis consume
them, so treat a field change there as an interface change here.

## Maintenance Checklist

When changing the integration:

1. Leave sitemap generation and build steps 01 through 04 under their existing
   owners unless the task explicitly concerns those systems.
2. Keep routine commands and configuration in the operator guide; keep design
   reasoning and invariants here.
3. Keep the project adapter and `Template/IndexNow/` behavior synchronized.
4. Change fingerprinting, sitemap parsing, HTTP behavior, and report types in
   the Rust binary with tests, not in this adapter.
5. Run shell syntax checks, `git diff --check`, the full local site pipeline,
   and `just IndexNow_Dry_Run`.
6. For live proof, inspect Woodpecker, the active target, public key, sitemap,
   and stored report.

