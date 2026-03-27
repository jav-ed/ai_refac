# Docs Site

The public-facing documentation for `refac` is a Starlight (Astro) static site. It lives in `docs-site/` at the repo root and is deployed to `refac.javedab.com` via Caddy.

## Links

- [Deploy Guide](./deploy_docs.md)
  Build command, rsync deploy, Caddyfile snippet, DNS setup.

## Content structure

All public pages live under `docs-site/src/content/docs/`. The directory layout maps directly to URL paths:

```
docs-site/src/content/docs/
├── index.mdx                          → refac.javedab.com/
├── getting-started/
│   ├── installation.md                → /getting-started/installation/
│   └── usage.md                       → /getting-started/usage/
├── languages/
│   ├── typescript.md                  → /languages/typescript/
│   ├── python.md                      → /languages/python/
│   ├── go.md                          → /languages/go/
│   ├── rust.md                        → /languages/rust/
│   ├── dart.md                        → /languages/dart/
│   └── markdown/
│       ├── index.md                   → /languages/markdown/
│       ├── supported-behavior.md      → /languages/markdown/supported-behavior/
│       ├── limits.md                  → /languages/markdown/limits/
│       └── examples.md                → /languages/markdown/examples/
├── reference/
│   └── capabilities.md               → /reference/capabilities/
└── development/
    ├── build-guide.md                 → /development/build-guide/
    └── testing.md                     → /development/testing/
```

## Adding a new page

1. Create a `.md` file in the appropriate directory under `docs-site/src/content/docs/`.
2. Add frontmatter at the top:
   ```md
   ---
   title: Page Title
   description: One-line description for SEO and search.
   ---
   ```
3. Add an entry to the `sidebar` array in `docs-site/astro.config.mjs`:
   ```js
   { label: 'Page Title', slug: 'section/page-filename' }
   ```
   The slug is the file path relative to `src/content/docs/`, without the `.md` extension. `index.md` files use the directory slug (e.g. `languages/markdown`, not `languages/markdown/index`).

## Adding a new language

1. Create `docs-site/src/content/docs/languages/<language>.md`.
2. Add a row to `reference/capabilities.md` (language support matrix + limits table).
3. Add the sidebar entry in `astro.config.mjs` under the Languages section.
4. Add a row to the quick reference table in `doc_Start.md`.

## Updating the sidebar

The sidebar is fully manual — defined in `docs-site/astro.config.mjs`. Edit the `sidebar` array directly. The `javedab.com ↗` entry at the bottom links to the services page and must remain the last item.

## Fonts

Both fonts are self-hosted variable woff2 files — no Google Fonts, no CDN dependency.

| Role | Font | Source |
|---|---|---|
| Body & headings | Geist (variable) | `Repos/geist-font/` → `public/fonts/Geist[wght].woff2` |
| Code blocks & inline code | Maple Mono (variable) | `Repos/maple-font/` → `public/fonts/MapleMono[wght]-VF.woff2` |

Applied via `src/styles/custom.css` using `--sl-font` and `--sl-font-mono` Starlight CSS variables. Expressive Code also receives `codeFontFamily: 'Maple Mono'` via `styleOverrides` in `astro.config.mjs`.

To update a font: replace the woff2 file in `public/fonts/` and update the `@font-face` src in `custom.css`.

## Build and verify

```bash
cd docs-site
bun run build
bun run dev   # preview at localhost:4321
```

## Relationship to internal docs

`docs-site/` is the **public** layer. `Project_Manag/Docs/` is the **internal** layer. They cover the same topics but serve different audiences:

| | Internal (`Project_Manag/Docs/`) | Public (`docs-site/`) |
|---|---|---|
| Audience | AI agents, developers | End users discovering the tool |
| Tone | Terse, reference-style | Welcoming, step-by-step |
| Purpose | Context for tasks, decision records | Discoverability, lead generation |

Keep them updated independently. When a language backend changes, update both the internal feature doc **and** the corresponding public language page.
