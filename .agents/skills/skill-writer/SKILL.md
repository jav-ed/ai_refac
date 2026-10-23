---
name: skill-writer
description: Create new agent skills using a navigation-first, domain-shaped structure. SKILL.md gives the agent orientation and common-case usage inline, then routes to purpose folders or references for the long tail. Use when the user wants to create, write, or build a new skill.
---

# skill-writer

A skill teaches an agent how to do something. The agent reads it alongside everything else in its current context, and must quickly decide which parts are relevant to the task at hand. So a skill is judged by how cleanly it surfaces the right information, not by how thoroughly it documents the topic. This file is itself structured the way it tells you to structure yours.

## The principle: navigation-first, not dump-first

An agent works best when it can quickly tell what in its context is relevant to the current task and what is not. Every irrelevant section loaded competes with the relevant ones and degrades the agent's decisions. So a skill must give the agent an easy time deciding what to read and what to skip. And what is not loaded must still be findable through good labels, since the agent cannot easily come back to a doc later. A skill must:

1. Give enough orientation upfront that the agent understands the domain.
2. Inline the common case so the most-used path needs no follow-up reads.
3. Route the long tail through clearly-labeled links, so the agent can decide what to load without opening the file. For a tool, name the command that lists the rest (`--help`, `just --list`) instead of copying its whole catalog.

The load-bearing piece is the labels, not the file split. A flat doc with sharp labels beats a deeply nested doc with murky ones every time.

## Skill structure

Use domain-shaped folders when the skill has clear purpose areas. Do not force meaningful areas into a generic `References/` bucket. `References/` is for miscellaneous supporting material that does not deserve a first-class domain folder.

```
skill-name/
├── SKILL.md                    # entry point: orientation + common case + links
├── Project_Onboarding/         # example domain folder: cohesive purpose area
│   └── first_Setup.md
├── Operations/                 # example domain folder: daily workflow / runbook
│   └── operations.md
├── Templates/                  # example asset folder: copyable files
├── Code/                       # optional executable helpers
└── References/                 # optional misc long-tail docs only
    └── background_Context.md
```

Top-level folders should be unified by purpose: onboarding, operations, architecture, templates, schemas, examples, code helpers, provider-specific integrations, and similar real domains. Add a folder when it improves readability, scannability, or scalability. Do not add a folder just to mirror a template.

Use `References/` only when the material is truly miscellaneous long-tail support. If a set of files has a coherent purpose, give that purpose its own folder instead of nesting it under `References/`.

If unsure about `.md` file or folder naming, see the `coding` skill, which documents the project-wide convention. (`SKILL.md` itself is uppercase, an exception to the lowercase rule because that is what the harness recognizes.)

A skill that fits the common case in 30 lines does not need a `References/` folder at all. Splitting is a tool, not a virtue.

## Audience separation: operator and developer

Many skills serve two readers:

- the operator agent needs to use an existing tool safely and complete the normal task;
- the developer or maintainer needs to change, debug, or extend how that tool works.

Keep the operator path in the parent page. It should contain the commands, required inputs, safety boundary, expected result, and actionable failures needed for ordinary use. Do not make the operator load implementation history, internal architecture, library rationale, schemas, or maintenance invariants.

When deeper developer material belongs to one operational area, place it in a singular `Detail/` folder beside that area's parent page:

```text
Application/Integration/
├── existing_Tool.md
└── Detail/
    └── existing_Tool_Architecture.md
```

The parent page must state the boundary near its opening and link with a decision-ready label such as:

```markdown
Do not load the developer detail for normal operation. Read
[Tool architecture](Detail/tool_Architecture.md) only when changing the
protocol, state machine, report contract, or implementation.
```

The detail page must reciprocate: open by saying it is developer background, name the exact change/debug tasks that require it, and route ordinary use back to the operator page.

`Detail/` is contextual long-tail material, not a mandatory folder for every skill. If architecture is a major first-class domain used independently across the skill, create a purpose-shaped `Architecture/` folder instead. If operator and developer information are always needed together, keep them together rather than splitting by title alone.

## Independence

A skill is copied into many repositories and read by an agent that knows nothing about the one it was written in. Test every link, command and path: does it still work after the folder is copied into an unrelated repository?

- No links or paths into one repository's docs, scripts or recipes, and no instruction to run a recipe only one repository defines (`just some-recipe`). Put the policy, the commands and the verification steps inside the skill.
- Machine facts may stay (hostnames, `/usr/local/bin`, device keys). Repository facts may not (folder layout, recipe names, project and customer names): those belong in that repository's docs.
- Name other skills ("the `coding` skill"), do not path-link them, and keep the common case completable without opening one. A pointer to a convention of the current project (`Project_Manag/Docs/doc_Start.md`) is fine.
- Examples use neutral names (`my-service`).

## Skills that need project customization

Some subjects vary per project: a design or quality bar, a product's voice, a content structure. Their skill gives the general guideline, and the repository's own rules must be able to win without anyone editing the skill, because a sync overwrites every project's copy. Such a skill carries the mechanism itself:

- a paragraph near the top of SKILL.md, "The repository wins", that names the override map at `Decisions/<Skill_Name>/linker_<Skill_Name>.md` under the project's docs root, says it is opened first for the folder being worked in, and says it overrides the skill where it speaks while the skill applies where it is silent;
- a `Repository/override_Map.md` page that explains the map, how to use it, and what belongs in the skill or in the repository; and, only when customising can break an agreement between files, a `Repository/what_Must_Hold.md`.

Say how firmly each piece of guidance holds, so a project knows what it may adapt: a *contract* (two files must agree, change both or neither), a *default* (follow it unless the project has a reason, and say why) or a *starting point* (a suggestion to adapt freely, with no record needed). Design, wording, values and motion are starting points; do not guide them harder than the skill can justify. Guidance that is not labelled is read as a rule, and projects then stop adapting it.

A skill that only documents a tool, a command or a fixed procedure does not need this. The test, the paragraph to copy, the page outline and the three strengths are in [Repository overrides](References/repository_Overrides.md).

## SKILL.md shape

Three layers, in order: orientation (one or two paragraphs on what the skill does and why), the common case (the most-used path, fully explained inline), and navigation (links to purpose folders or references for the long tail).

````markdown
---
name: skill-name
description: One sentence on what the skill does. Use when [specific triggers].
---

# skill-name

[Orientation: what the skill does, why an agent would reach for it, framing.]

## [Common-case section heading]

[Inline content for the most-used path. Multiple ## sections are fine.]

## Navigation

- [Project onboarding](Project_Onboarding/first_Setup.md): description rich enough for the agent to decide without clicking
- [Operations](Operations/operations.md): same
- [Background context](References/background_Context.md): misc reference only when no better purpose folder exists
````

Frontmatter: max 1024 chars, third person, two sentences (what it does, when to use it). Quote the description when it contains `: `, otherwise strict YAML parsers reject the file. The description is the only thing the agent sees when deciding whether to load the skill, so vague phrasing kills it. The limit is a maximum, not a target: procedures, command lists and gotchas belong in the body.

Link descriptions are the load-bearing piece. The text after the colon must say what kinds of tasks or questions belong behind the link, rich enough that the agent can decide whether to follow without opening the file. A bare filename or topic name is not a description. Length follows need: one line when one line covers it, five lines when the topic genuinely needs five. Optimize for clarity, not brevity.

See [Labels and descriptions](References/labels.md) for examples and failure modes.

## Recursion

A reference file is a smaller skill. It gets its own orientation, common case, and links if it needs them. The pattern applies at every level. If a reference file turns into a dump, split it the same way SKILL.md was split.

## Review checklist

Before declaring the skill done:

- Frontmatter `name` matches the folder name; `description` is under 1024 chars, third person, two sentences with a specific "Use when" trigger.
- SKILL.md opens with orientation, fully covers the common case inline, and routes the long tail through clearly-labeled links to purpose folders or references.
- Every link uses `[Label](path.md): description` format with a colon. No bare filenames, no vague "see also".
- Folder structure is domain-shaped: coherent purpose areas are top-level folders, not buried under `References/`.
- `References/` is absent unless there is real miscellaneous long-tail material.
- Each linked reference or domain file follows the same orientation + common case + links pattern. No linked file is a dump.
- Operator pages contain normal usage without requiring developer internals; contextual maintainer material lives under singular `Detail/` with explicit read/skip labels in both directions.
- Independent: the common case still works after the folder is copied into an unrelated repository.
- If the subject varies per project (design, voice, structure), the skill has the "repository wins" paragraph and a `Repository/override_Map.md`; if it only documents a tool or a fixed procedure, it has neither. Guidance that is only a suggestion is labelled a starting point.
- Self-test: if the agent only reads SKILL.md, can it do the common case end-to-end?

## References

- [Process](References/process.md): drafting a skill from scratch with the user, gathering requirements, reviewing, iterating.
- [Labels and descriptions](References/labels.md): rules for the frontmatter description and inline link descriptions, with format, character limits, length guidance, and worked good/bad examples.
- [Failure modes](References/failure_Modes.md): over-splitting, misjudged common case, atomization, premature linker creation, and other ways the navigation-first pattern gets misapplied.
- [Repository overrides](References/repository_Overrides.md): when a skill needs a place for the project's own rules, the paragraph to copy into SKILL.md, the outline of the override page and what-must-hold page, and what the repository's override map contains. Read it when the skill's subject varies per project, such as design, voice or content structure.
- [When to add code](References/code.md): criteria for bundling executable helpers in the skill, and when generated code is fine.
