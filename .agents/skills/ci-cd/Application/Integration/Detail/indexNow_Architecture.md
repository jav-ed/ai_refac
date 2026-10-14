# IndexNow Architecture And Maintenance

Use this developer detail only when changing IndexNow behavior or diagnosing an
invariant failure. For installation, configuration, dry runs, recovery, and
normal deployment, use the [IndexNow operator guide](../indexNow_Notifications.md).

## Contents

- [Ownership boundaries](#ownership-boundaries)
- [Sitemap boundary](#sitemap-boundary)
- [Release comparison](#release-comparison)
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

The separate Rust `indexnow` utility owns:

- sitemap-index and leaf-sitemap parsing;
- URL, origin, path, and document validation;
- semantic HTML fingerprints and release comparison;
- ownership verification and HTTP submission;
- typed comparison and submission reports.

Woodpecker calls shell steps directly. Just exposes human-facing commands and is
not a CI dependency.

## Sitemap Boundary

Validation is not a second sitemap pipeline. It is a safety check on the exact
artifact that IndexNow is about to use.

For each release, the Rust utility:

1. Reads the configured sitemap entry file produced by Astro.
2. Resolves same-origin leaf sitemaps when the entry is a sitemap index.
3. Maps each sitemap URL to a page inside that release root.
4. Rejects malformed XML, external origins, traversal, and missing pages.
5. Uses those URLs as the public comparison allowlist.

This preserves Astro's ownership while preventing IndexNow from submitting an
incomplete, malformed, or unrelated URL set. Removed URLs remain detectable
because the previous release has its own sitemap-approved set.

## Release Comparison

Compare the exact release that was active before deployment with the exact
release activated by the current deployment. Never infer ancestry from the
second-newest directory; manual activation and rollback can change that order.

Source-file diffs are insufficient because shared layouts, navigation, schema,
or rendering can change many public pages. Raw HTML bytes are also too noisy
because formatting, minification, classes, and generated asset identifiers can
change without changing page meaning.

The Rust fingerprint therefore represents search-relevant content such as:

- title, description, canonical, hreflang, and robots metadata;
- structured data;
- headings and visible body text;
- link destinations;
- meaningful image sources and alternative text.

It excludes ordinary formatting and generated presentation noise. URLs are
classified as `added`, `changed`, `removed`, or `unchanged`; only the first
three classes are submission candidates. Fingerprint changes belong in the
Rust utility with focused fixtures and tests.

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

Reports live on the deployment host beside the lane's releases, outside the
Caddy-served `dist/` tree and outside Git.

Consumers rely on:

- comparison counts and URL classifications;
- submitted URL count;
- public-key verification state;
- submission batches and HTTP results;
- final report placement through atomic rename.

Treat field or invariant changes as interface changes because bootstrap state,
operator summaries, recovery, and diagnosis consume this data.

## Maintenance Checklist

When changing the integration:

1. Leave sitemap generation and build steps 01 through 04 under their existing
   owners unless the task explicitly concerns those systems.
2. Keep routine commands and configuration in the operator guide; keep design
   reasoning and invariants here.
3. Keep the project adapter and `Template/IndexNow/` behavior synchronized.
4. Change fingerprinting, sitemap parsing, HTTP behavior, and report types in
   the Rust utility with tests.
5. Run shell syntax checks, `git diff --check`, the full local site pipeline,
   and `just IndexNow_Dry_Run`.
6. For live proof, inspect Woodpecker, the active target, public key, sitemap,
   and stored report.

