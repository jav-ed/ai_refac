# Index_Now

A Rust CLI (`indexnow`, run as `Index_Now_Bin`) that tells the IndexNow network which pages of a static site really changed. It compares the new release with the previously deployed one through their sitemaps, fingerprints the visible content and search metadata of every listed page, and submits only the URLs that changed. A markup-only change does not count as a change.

## When to add it

IndexNow is opt-in: do not add it to every site. The site needs all of these:

- a retained previous release next to the active one, and an active-release symlink (the layout of the deploy step);
- an HTTPS public origin;
- a sitemap, preferably `sitemap-index.xml` linking leaf sitemaps (never flatten the index). The site's framework generates it; Index_Now only reads the finished file and never repairs it. A site without a sitemap cannot use Index_Now.
- an ownership key: a file `<key>.txt` at the site root whose content is exactly `<key>` (one final newline allowed). It is public, not secret, and it proves submission authority for the host. Astro copies it from its `publicDir`, so a custom `publicDir` must hold the file.

## Commands

`indexnow --help` and `indexnow <command> --help` list every flag.

| Command | Network | Use |
| --- | --- | --- |
| `inspect-sitemap <file>` | no | Parse one local sitemap file and print it as JSON. |
| `fingerprint <file>` | no | Print the semantic model and SHA-256 fingerprint of one HTML file. |
| `compare-page` | no | Compare two HTML files for one URL and say whether a submission would be needed. |
| `compare-releases --site-url --sitemap --previous-root --current-root [--extensionless-page-urls] [--report]` | no | Compare two whole releases and report added, changed, removed and unchanged URLs. The safe dry run. |
| `submit-releases` (same flags plus `--key-file`, `--key-location`, `--endpoint`, `--timeout-seconds`) | **yes** | Compare, verify the key file is publicly served with the exact content, then submit only the changed URLs. |

`--site-url` includes the scheme and a trailing slash; `--sitemap` is relative to both release roots. A sitemap page URL must end in `/` or `.html`; a site whose URLs have neither (`/docs/usage`) passes `--extensionless-page-urls` to `compare-releases` and `submit-releases`, which states that its host serves them as `<path>/index.html`. The default endpoint is `https://api.indexnow.org/indexnow` and the default timeout is 15 seconds.

## What the binary does when something is wrong

- It never guesses: no alternate release, sitemap, key path or endpoint is tried.
- HTTP redirects are off. A rate limit is not retried blindly: the error carries `Retry-After` when the server sent one.
- A deployment with no changed pages makes no network request at all, not even the key fetch.
- Before the first POST it fetches the public key URL and requires the exact content, so a stale cache, the wrong release or a routing error stops the submission.
- An accepted response means IndexNow received the URLs, not that anything was crawled or indexed.

## In a pipeline

A site calls the binary from its deploy step through a small shell adapter: the overlay files, the explicit `CI_INDEXNOW_*` variables, the bootstrap state on the deployment host, the order of the deploy, the recovery commands, and the verification before the first live run. All of that belongs to the pipeline and is in the `ci-cd` skill, on its IndexNow notifications page. The binary itself is built on this machine and installed on the host by its own pipeline (`Index_Now_Bin` in [path_File.md](../path_File.md)).

## Deeper

Two places, split by what each one describes. For changing the tool (sitemap parsing, fingerprints, comparison, submission, reports): `Project_Manag/Docs/doc_Start.md` in `Index_Now_Home`, especially `Architecture/processing_Pipeline.md` (module ownership, why fingerprints are semantic, what sitemap validation is for, the report contract) and `Architecture/woodpecker_Integration.md` (installing the binary, the order a site's deploy must keep). For changing how a site's deploy calls it (bootstrap state, the deployment transaction, locking, retention order): the `ci-cd` skill's adapter contract page, `Application/Integration/Detail/indexNow_Architecture.md`, which sits next to the overlay code it describes. Local development of the tool: `just check`, `just deploy` and `just publish` in `Index_Now_Home`.
