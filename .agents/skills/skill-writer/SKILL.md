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

## Shipping a change

A skill lives in a vault, a git-tracked folder separate from any project, that `skillmirror`, a small CLI tool, copies into every project's `.agents/skills/`. No project edits the vault directly; the order protects every other project from an unreviewed change:

1. **Edit the project's own installed copy first.** Try the change where it is actually read, and get it approved there before anything leaves that project.
2. **Replace the vault's copy completely, never merge into it.** `skillmirror promote <skill-name>` removes the vault's existing folder and puts the project's in its place (kept as a backup, staged in git, never committed). Copying only the changed files over the top leaves behind whatever the edit deleted or renamed: a page the new structure no longer links to would keep sitting in the vault, and then in every project the next sync reaches.
3. **Push it out.** `skillmirror sync <skill-name>` updates every other project that already has the skill; it never adds the skill somewhere new. `promote --sync` does both steps together, asking again before the sync runs.

A brand-new skill starts with `skillmirror new <name>` (a template, staged in git) or `skillmirror adopt <name> --from <project>` (a skill a project already wrote, brought into the vault for the first time). `skillmirror init <dir> [skills...]` creates a new project with the vault's mandatory skills plus a chosen selection. `skillmirror diff` and `skillmirror status` preview what a sync would change before anything changes. The full command reference is in [Shipping skills with skillmirror](References/skillmirror.md).

## Skills that need project customization

Some subjects vary per project: a design or quality bar, a product's voice, a content structure, a cost or benefit bar. Their skill gives the general guideline, and the repository's own rules must be able to win without anyone editing the skill, because a sync overwrites every project's copy. Such a skill names, on the page that carries the overridable guidance (SKILL.md itself, or the one sub-page that owns the topic), one exact path where a repository's override would live. Choose that path to fit the skill's own subject and how a project installing it already organizes its own docs, never a fixed `Decisions/<Skill_Name>/linker_<Skill_Name>.md` convention applied regardless of fit: a testing page reuses whatever folder a project already uses for tooling docs; a design or voice skill more often fits under a project's `Decisions/`, where approved product choices already live.

State the whole mechanism in one paragraph, wherever the path is named: the exact path; open it before relying on the skill's guidance for that topic; what it names overrides the skill where it speaks, and the skill applies where the document is silent; and, stated plainly rather than left implicit, that the path or file not existing means nothing is overridden, not an error and not a reason to search further.

A skill whose overridable facts are scattered across many of its own pages still names only one path, at the skill level, in SKILL.md; the document living there may be an index with one section per skill folder, since that is real content the project owns, not mechanism to explain. A skill with a single overridable fact confined to one page names its path inline, on that page, in a sentence or two. Neither case needs a dedicated page whose only job is to explain how the mechanism works. Only when customising can break an agreement between files (a name two scripts share, a number two stylesheets repeat) does the skill also carry a `Repository/what_Must_Hold.md`: real content (a table of contracts), not mechanism explanation, so it keeps its own page.

Say how firmly each piece of guidance holds, so a project knows what it may adapt: a *contract* (two files must agree, change both or neither), a *default* (follow it unless the project has a reason, and say why) or a *starting point* (a suggestion to adapt freely, with no record needed). Design, wording, values and motion are starting points; do not guide them harder than the skill can justify. Guidance that is not labelled is read as a rule, and projects then stop adapting it.

A skill that only documents a tool, a command or a fixed procedure does not need this. The test, the paragraph to write, and worked examples at both scales, are in [Repository overrides](References/repository_Overrides.md).

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
- If the subject varies per project (design, voice, structure, a cost or benefit bar), the page that carries the overridable guidance names one exact path, chosen to fit the skill, with the full mechanism in one paragraph; if it only documents a tool or a fixed procedure, it names none. Guidance that is only a suggestion is labelled a starting point.
- Self-test: if the agent only reads SKILL.md, can it do the common case end-to-end?

## References

- [Process](References/process.md): drafting a skill from scratch with the user, gathering requirements, reviewing, iterating.
- [Shipping skills with skillmirror](References/skillmirror.md): the full command set for moving a skill between a project and the vault and out to every other project: `promote`, `sync`, `adopt`, `new`, `init`, and the read-only `diff`, `status`, `changes`, `doctor`, `info`, `undo`. Read it before promoting a change, adopting a project's skill into the vault, or bootstrapping a new project.
- [Labels and descriptions](References/labels.md): rules for the frontmatter description and inline link descriptions, with format, character limits, length guidance, and worked good/bad examples.
- [Failure modes](References/failure_Modes.md): over-splitting, misjudged common case, atomization, premature linker creation, and other ways the navigation-first pattern gets misapplied.
- [Repository overrides](References/repository_Overrides.md): when a skill needs a place for the project's own rules, one named path chosen to fit the skill, and the two scales (a single inline fact versus facts scattered across many pages). Read it when the skill's subject varies per project, such as design, voice or content structure.
- [When to add code](References/code.md): criteria for bundling executable helpers in the skill, and when generated code is fine.
