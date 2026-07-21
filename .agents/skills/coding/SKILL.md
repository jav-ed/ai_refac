---
name: coding
description: Coding conventions — commenting style, naming, structure, and universal rules. Use whenever writing or reviewing code in any language.
---

# Coding Standards

## Mandatory Universal Rules — Read Before Coding

These rules apply to every language and every coding task. They are not optional. Read them before consulting language-specific guidance.

- **Obvious code**: make inputs and outputs crystal clear. Avoid unnecessary abstractions. Keep methods, classes, and functions straightforward.
- **Single responsibility**: one file, one clear purpose. If a function does not clearly belong anywhere, place it in a `helpers` file within the relevant feature folder.
- **No deep nesting**: use early returns and guard clauses to keep logic flat.
- **Maximum 300 lines of code per file**: comments do not count toward this limit. Comments must never be removed merely to reduce the line count.
- **Never remove comments**: preserve comments unless the behavior they describe has changed or their removal was explicitly requested. When behavior changes, update the relevant comment.
- **No fallbacks**: hard breaks only. Missing, malformed, unsupported, or contradictory data must cause an explicit failure. Do not silently substitute defaults or conceal broken contracts. If things crash, we need to experience those crashes.

## Commenting

Comments preserve design intent, repository rules, architectural boundaries, and non-obvious reasons behind an implementation. They should help a future developer understand not only what the code does, but why it must be done that way and when the same approach should be reused.

Add comments above:

- architectural or performance boundaries;
- intentionally eager or lazy-loaded code;
- custom validation and product rules;
- unusual cleanup, derivation, or compatibility logic;
- reusable patterns whose purpose is not obvious from the syntax.

Do not add comments that merely narrate obvious code.

### Example: reusable lazy-loading boundary

```ts
// Keep route validation and authorization eager so access rules run before
// rendering. Lazy-load only the route UI; use this boundary for other protected
// screens so feature code does not enter the shared application bundle.
const Route_View = lazy(() => import("./View"));
```

Place a comment directly above the code it describes. Separate the next logical block with a blank line:

```
// comment here
first_code

// second comment here
second_code
```

## File and Folder Structure

The file tree is part of the codebase's interface. A clear tree lets developers understand the system, locate behavior quickly, and extend it without searching through unrelated files.

Organize primarily by **feature**, not by technical type. Each feature folder should expose one clear entry file, with narrowly focused supporting files beside it. You should be able to read the entry file to understand what the feature does, then follow its imports for implementation details.

Every path segment must add useful information. A child name must not repeat context already supplied by its parent folder.

```text
Profile/
  Editor.tsx             ← feature entry
  validation.ts          ← supporting file
  helpers.ts             ← supporting file
  types.ts               ← supporting file

Imgs/
  hero.webp
  optimization_Guide.md
```

Do not write redundant paths such as:

```text
Imgs/
  Img_Hero.webp
  Img_File_New.md
```

The `Imgs/` parent already provides the image context. Repeating `Img` makes the tree noisier without adding meaning.

- Name files according to their local responsibility, not their entire ancestry.
- A folder should represent one cohesive domain, feature, or meaningful subdivision.
- One file has one clear purpose. If a file owns two distinct responsibilities, split it.
- Split a feature into a subfolder when its supporting files become difficult to scan. Keep every file within the 300-line code limit and preserve its comments.
- Do not create layers or folders that contain only one trivial forwarding file.
- Shared or cross-cutting code gets its own clearly named folder, such as `shared/`, `utils/`, or `core/`.
- A dedicated `helpers` file may contain small functions that do not clearly belong elsewhere within the feature. Do not create a separate folder for every small helper.

## Naming Conventions

**Step 1 — check if the codebase already has an established convention.** If it does, follow it exactly. Do not introduce a different style, even if it looks better.

**Step 2 — if no convention exists, ask the operator:**

> "Which naming style should I use?
> - **Standard** — the default community norms for this language (camelCase/kebab-case for JS, snake_case for Python). Good when an AI agent is writing the code autonomously.
> - **Improved** — a more readable hybrid that keeps underscores for separation but uses capitalization to signal nouns and significant concepts. Better when the developer is reading the code themselves."

## Language-Specific Guidance

Read the relevant language guide only after the mandatory universal rules. Language-specific conventions may refine naming and syntax, but they do not override the universal rules.

- [JS / TS](languages/js_Ts.md) — when the repository uses standard JS/TS naming.
- [Markdown and non-code files](languages/markdown.md) — for documentation and other non-code files.
- [Improved_Camel_Snake convention](languages/improved_Version.md) — when the repository already uses it or the operator selects it.
