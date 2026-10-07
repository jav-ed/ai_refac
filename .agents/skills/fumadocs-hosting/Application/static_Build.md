# Static Build

## Required Contract

- Derive TanStack `pages` from the Fumadocs content files so new pages enter the build automatically.
- Prerender every public docs URL as HTML; retain `_shell.html` only as a fallback.
- Run Pagefind after prerendering so it indexes the real pages.
- Split oversized framework chunks by stable package boundary; do not merely raise the warning limit.

## Verification

```bash
bun run types:check
bun run build
find dist/client -type f -name '*.html' -print
```

The expected count is all public pages plus `_shell.html`. Test a deep URL through the same static-server fallback used by Caddy.
