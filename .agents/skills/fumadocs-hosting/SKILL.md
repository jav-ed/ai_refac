---
name: fumadocs-hosting
description: Prepare and verify static Fumadocs sites built with TanStack Start for Caddy hosting. Use this skill for prerendering, Pagefind, precompression, cache rules, CI delivery, and production handoff.
---

# Fumadocs Hosting

Build a static artifact first; production activation remains a separate, explicit step.

## Common Workflow

1. Identify the framework versions, build command, and static output directory.
2. Ensure every public content route is explicitly prerendered. Wildcard routes are not proven by a shell-only build.
3. Run type checking and the complete production build, including Pagefind.
4. Confirm real HTML exists for deep routes and no client chunk exceeds the agreed budget.
5. Precompress the final static directory.
6. Validate Caddy and deep links locally.
7. Commit the build, CI, and tracked Caddy configuration separately from production activation.

Never publish `dist/server` for a static deployment. For TanStack Start SPA builds, the usual deployable directory is `dist/client`.

## Navigation

- [Static build](Application/static_Build.md) — explicit prerender routes, Pagefind, and bundle checks.
- [CI/CD](Application/ci_CD.md) — pipeline stages and atomic release contract.
- [Caddy](Platform/caddy.md) — cache, compression, fallback, and validation rules.
- [Handoff](Application/handoff.md) — the production colleague checklist.
