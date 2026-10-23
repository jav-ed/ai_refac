# Repository overrides

How to give a skill a place for the project's own rules, so the skill stays general and a project can still win where it differs. Use it when writing a skill whose subject varies per project. This is a starting point: revise it when a real skill shows what is missing or too long.

## Does the skill need it

Ask: would two projects using this skill reasonably make different choices? If yes, the skill is a guideline and needs the mechanism. If no, it does not.

- Needs it: a design or quality bar, a product's voice and copy, a content structure, a framework convention with per-project choices (a blog, a component library).
- Does not need it: how to use a tool or a command, a fixed procedure, a protocol.

The reason is a sync. A skill is copied from the vault into every project, and the next sync overwrites that copy. A project that edits its copy to fit itself loses the edit, and a project-specific rule put into the vault reaches every other project. So project rules live in the project's docs, and the skill says where.

## What the skill contains

1. **A paragraph near the top of SKILL.md.** Copy this and fill in the name (`Skill_Name` is the skill's name in Title_Case with underscores, `astro-blog` becomes `Astro_Blog`):

   ```markdown
   **The repository wins.** This repository's own rules sit in an override map at `Decisions/Skill_Name/linker_Skill_Name.md` under its docs root. Open the map's section for the folder you are working in before you rely on a page of this skill; what the map names overrides this skill for that topic, and where it is silent this skill applies. [The repository's rules](Repository/override_Map.md) explains the map.
   ```

2. **`Repository/override_Map.md`.** One page with three parts, kept as short as the skill allows:
   - *Where they live:* the map's path, what it opens with (a summary of how this repository uses the subject), and its entries. One section per folder of the skill. Each entry is `[Label](path.md): what it decides, and whether it overrides or adds to the skill's page`. It links to the document that owns the rule and never copies it, so every fact has one home. A rule with no owning document yet gets a page in the same folder, named like the skill page it overrides. The map records contracts that were changed, defaults that were departed from, and rules the skill is silent on; a changed starting point needs no entry of its own.
   - *How to use it:* open the map's section before relying on a skill page; a document the map names wins for its topic; where the map is silent the skill applies; say so in the report when a point cannot be met; without a map treat the skill as the default and, when the user states a rule that belongs to the repository, offer to record it in the map and never edit the skill's copy; a rule right for every project goes into the skill through the vault.
   - *Skill or repository:* a table of what stays in the skill (guidance on how to do it right) and what stays in the repository (facts, names, values, code), plus the line "keep out of the skill: a project's own names, paths and word lists; examples stay neutral". Keep this a table inside the page, not a page of its own.

3. **`Repository/what_Must_Hold.md`, only when needed.** If customising the skill's subject can break an agreement between files (a name two scripts share, a number two stylesheets repeat), add one page that lists, per area, what must change in every place at once next to what is free to choose. Skills whose customisations cannot break each other skip it.

Link the override page from the SKILL.md navigation, and have the frontmatter description's "Use when" include the case of a project needing its own rules.

## How firmly guidance holds

Label guidance by strength, so a project does not treat a suggestion as a law or a contract as a suggestion:

- **Contract:** two files must agree on a name or a number. Listed in `Repository/what_Must_Hold.md`; changed everywhere at once, and the change is recorded in the map.
- **Default:** follow it unless the project has a reason. A departure is recorded in the map with its reason.
- **Starting point:** a suggestion the project adapts as far as it needs, with no record just for changing it. The project's own docs own the result (a brand page for a look, an architecture page for a pipeline); the map may route to them but does not call them deviations.

Design, wording, values (sizes, colours, timings) and motion are starting points unless something breaks without them. Say it once, in a short line at the top of the page or section ("Starting point: change freely"), or in the navigation entry. Guide less rather than more: a skill that fixes what it cannot justify makes projects fight it, and a starting point that is right will be kept anyway.

## What the repository contains

The map is a linker at `Decisions/Skill_Name/linker_Skill_Name.md` under the docs root, found through the repository's doc start and routed from `Decisions/linker_Decisions.md`. It is created when the repository gets its first rule for the skill, not before: an empty map is noise. The documents it links to stay where their subject lives (Architecture, Brand, Legal, and so on); the map only routes to them. See the `doc-start` skill for linkers and labels.

## Worked example

The `astro-blog` skill (where installed) has the paragraph, the override page and a what-must-hold page. Its repository-side map lists, per skill folder, which project documents override or add to it.
