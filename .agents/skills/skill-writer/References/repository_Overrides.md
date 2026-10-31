# Repository overrides

How to give a skill a place for the project's own rules, so the skill stays general and a project can still win where it differs. Use it when writing a skill whose subject varies per project. This is a starting point: revise it when a real skill shows what is missing or too long.

## Does the skill need it

Ask: would two projects using this skill reasonably make different choices? If yes, the skill is a guideline and needs the mechanism. If no, it does not.

- Needs it: a design or quality bar, a product's voice and copy, a content structure, a framework convention with per-project choices (a blog, a component library), a cost or benefit bar that should scale with what a project's own checks actually cost.
- Does not need it: how to use a tool or a command, a fixed procedure, a protocol.

The reason is a sync. A skill is copied from the vault into every project, and the next sync overwrites that copy. A project that edits its copy to fit itself loses the edit, and a project-specific rule put into the vault reaches every other project. So project rules live in the project's docs, and the skill says exactly where.

## One named path, chosen to fit

Pick the path a repository's override would live at, and name it directly on the page that carries the overridable guidance. Fit the path to the skill's subject and to how a project installing this skill already organizes its own docs, never a fixed `Decisions/<Skill_Name>/linker_<Skill_Name>.md` convention applied regardless of fit. A testing page can reuse a project's existing tooling-docs folder; a design or voice skill more often fits under a project's own `Decisions/`, where approved product choices already live. There is no single right folder name across skills: ask what a project maintaining this skill's subject would already call the place, and use that.

State the whole mechanism in one paragraph, wherever the path is named:

1. the exact path;
2. open it before relying on the skill's guidance for that topic;
3. what it names overrides the skill where it speaks, and the skill applies where the document is silent;
4. the path or file does not exist: nothing is overridden, not an error, use the skill's own guidance as written.

Point 4 is the part a mechanism most often leaves out, and the one a skill must say plainly: a missing file ends the check, it does not send the agent looking elsewhere.

## Two scales, not two mechanisms

- **A single overridable fact, confined to one page** (a cache-benefit bar on a testing page): name the path inline, in a sentence or two, on that page. No separate page for the mechanism, no paragraph at the skill's top level. `coding`'s testing page is the worked example: "A repository's own numbers, if any, live under its own `Tooling/` docs folder — check `Tooling/linker_Tooling.md` first. Not there? Nothing is overridden; the default above applies."
- **Facts scattered across many of a skill's own pages** (a design or quality bar, a product's voice, touching most of the skill's folders): name one path at the skill level, in a paragraph near the top of SKILL.md. The document living at that path may be an index with one section per skill folder, each entry linking to the project document that owns a fact and saying whether it overrides or adds to the skill's page. That index is real content the project owns, not boilerplate explaining how to read it, so it earns its place; what does not earn a page of its own is a second, skill-side page whose only job is to restate the four points above. Fold those into the one paragraph instead.

Either scale may also carry a `Repository/what_Must_Hold.md`, only when customising can break an agreement between files (a name two scripts share, a number two stylesheets repeat): a table of contracts, per area, next to what stays free to choose. This is real content too, so it keeps its own page; link it directly from wherever the path is named.

Link whichever pages survive from the SKILL.md navigation, and have the frontmatter description's "Use when" include the case of a project needing its own rules.

## Skill or repository

Keep a short, standing test for what belongs on which side, inline or as its own page when it is long enough to earn one: guidance on how to do something right stays in the skill; a fact, a name, a value, or the code that does it stays in the repository. Keep out of the skill: a project's own vocabulary, word lists, product names, file paths, error codes and code; examples stay neutral.

## How firmly guidance holds

Label guidance by strength, so a project does not treat a suggestion as a law or a contract as a suggestion:

- **Contract:** two files must agree on a name or a number. Listed in `Repository/what_Must_Hold.md`, if the skill has one; changed everywhere at once, and the change is recorded at the named path.
- **Default:** follow it unless the project has a reason. A departure is recorded at the named path with its reason.
- **Starting point:** a suggestion the project adapts as far as it needs, with no record just for changing it. The project's own docs own the result (a brand page for a look, an architecture page for a pipeline); the named path may route to them but does not call them deviations.

Design, wording, values (sizes, colours, timings) and motion are starting points unless something breaks without them. Say it once, in a short line at the top of the page or section ("Starting point: change freely"), or in the navigation entry. Guide less rather than more: a skill that fixes what it cannot justify makes projects fight it, and a starting point that is right will be kept anyway.

## Worked examples

- `coding`, testing page: a single fact (the cache-benefit bar), named inline on the page that states the default, in two sentences, no separate pages at all.
- `web-app-quality` and `astro-blog`: facts scattered across most of the skill's folders, named once in SKILL.md's opening paragraph, pointing at a project-side index (an actual `Decisions/<Skill_Name>/linker_<Skill_Name>.md` in the projects that install them, since that is where each of those projects already keeps approved product decisions); `Repository/what_Must_Hold.md` stays where customising their subject can break an agreement between files.
