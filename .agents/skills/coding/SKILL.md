---
name: coding
description: Coding conventions — commenting style, naming, structure, testing stance, and universal rules. Use whenever writing or reviewing code in any language.
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
- **Justify every test**: do not create a test you cannot justify. Handling tests eats the time that building needs. Read [Testing](Tooling/Testing/testing.md) before creating or running any test.

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

## Testing

If you create, run, or change a test, you must read [Testing](Tooling/Testing/testing.md) first.

## Searching code

Before you change code you do not know, find its definitions, callers and implementors with the `search` skill.

## File and Folder Structure

Group related work by purpose so developers can easily find and understand it. Two skills can help here: `file-tree-optimization` for grouping rules, ownership, naming, and inspection commands, and `refac-cli` for moving or renaming files, folders, and symbols with their references updated, in the languages it supports. They work well together: first decide where things belong, then move them.

## Naming Conventions

**Step 1 — check if the codebase already has an established convention.** If it does, follow it exactly. Do not introduce a different style, even if it looks better.

**Step 2 — if no convention exists, ask the operator:**

> "Which naming style should I use?
> - **Standard** — the default community norms for this language (camelCase/kebab-case for JS, snake_case for Python). Good when an AI agent is writing the code autonomously.
> - **Improved** — a more readable hybrid that keeps underscores for separation but uses capitalization to signal nouns and significant concepts. Better when the developer is reading the code themselves."

## Language-Specific Guidance

Read the relevant language guide only after the mandatory universal rules. Language-specific conventions may refine naming and syntax, but they do not override the universal rules.

- [Languages](Languages/linker_Languages.md): naming conventions (standard JS/TS, Improved_Camel_Snake, Markdown and non-code files) and language-specific knowledge such as JS/TS testing. Open it when you write or review code in a language it lists, or when you pick a naming style.
