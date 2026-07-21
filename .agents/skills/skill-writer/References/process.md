# Process

Steps for drafting a new skill from scratch with the user.

## 1. Gather requirements

Ask the user:

- What task or domain does the skill cover?
- Which use cases must it handle, and which is the most common?
- Does it need executable helpers (deterministic operations) or only instructions?
- Are there reference materials, examples, or existing docs to bundle?

The most common use case is what gets inlined in SKILL.md. If the user cannot name a clear common case, the scope is probably too broad to be one skill.

## 2. Draft

Create:

- `SKILL.md` with frontmatter, orientation, common-case content, and link descriptions for the long tail.
- Domain-shaped top-level folders for cohesive purpose areas, such as `Operations/`, `Project_Onboarding/`, `Templates/`, `Schemas/`, or provider-specific integrations.
- `References/<topic>.md` only for miscellaneous long-tail topics that do not belong to a clearer purpose folder.
- `Code/` only if the skill needs deterministic helpers. See [When to add code](code.md).

Write the domain files first if the structure is unclear. What survives in inline form is usually obvious once the long tail is on the page.

## 3. Review with the user

Before declaring done, ask:

- Does the inline content cover the common case you described?
- Is anything missing, or anything inline that belongs in a reference?
- Are the link descriptions specific enough to decide without clicking?

Then verify against the review checklist in SKILL.md.
