---
name: coding
description: Coding conventions — commenting style, naming, structure, and universal rules. Use whenever writing or reviewing code in any language.
---

# Coding Standards

## Commenting

Commenting is critical. Comments explain what we are doing AND why we are doing it that way — capturing mindset and design decisions. This helps spot shortcomings while coding and lets other devs understand the work.

Comment style — the comment sits directly above the code it describes, separated from the next block by a blank line:

```
// comment here
first_code

// second comment here
second_code
```

## Naming Conventions

Follow the native conventions of the language you are working in. For languages where the community is split or conventions are ambiguous, see the language guides:

- [JS / TS](languages/js_Ts.md)
- [Markdown & non-code files](languages/markdown.md)

## File & Folder Structure

Ideal strucutre, which you might not be able to always obtain: Organize by **feature**, not by type. Each feature gets its own folder with one clear entry file — this is the face of the feature. All supporting code (helpers, types, queries, etc.) lives in subfiles alongside it. You read the entry file and know what the feature does; you follow the links for implementation detail.

```
user/
  user.ts       ← entry file — public interface of the feature
  queries.ts    ← supporting file
  helpers.ts    ← supporting file
  types.ts      ← supporting file
```

- One file, one clear purpose. If a file is doing two things and it is already big, try to get two files out of it.
- Split into a subfolder whenever a group of subfiles grows large enough to warrant it (max 300 lines per file, keep comments).
- Shared/cross-cutting code gets its own clearly named folder (e.g. `shared/`, `utils/`, `core/`).
- A dedicated `helpers` file is valid — it can contain small functions that don't clearly belong anywhere else and don't need to share the same purpose. Creating subfolders for every small function is overkill.

## Universal Rules — All Languages

- **Obvious code**: make inputs and outputs crystal clear. Avoid unnecessary abstractions — keep methods, classes, and functions straightforward.
- **Single responsibility**: one file, one clear purpose. If a function doesn't clearly belong anywhere, it goes in a `helpers` file within the relevant feature folder.
- **No deep nesting**: use early returns and guard clauses to keep logic flat.
- **Max 300 lines per file** — comments do not count toward this limit and cannot be removed to reach it.
- **Never remove comments** — comments stay unless the logic they describe has changed or removal is explicitly requested.
- **No fallbacks** — hard breaks only. If things crash, we need to experience those crashes.
