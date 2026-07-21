# Failure modes

Ways the navigation-first pattern gets misapplied.

## Over-splitting

Two pieces that are always used together should not be in separate files. Splitting them taxes every use with an extra link-follow. Cohesion matters more than uniform file size. If you find yourself opening files A and B together every time, merge them.

## Misjudged common case

Inlining what feels important rather than what is actually common pollutes context for the majority. The "common case" is the path used in most invocations, directionally somewhere in the 60-80%+ range, not whatever the author considers the headline feature. Before inlining, ask: in what fraction of uses will the agent actually need this? If it is a minority case, route it.

## Atomization

A skill broken into ten one-paragraph reference files forces ten routing decisions. Each is overhead with no payoff when the topics belonged together. Group related material into fewer, larger reference files when the topics travel together.

## Premature linker creation

A 30-line skill that fits the common case inline does not need a `References/` folder. Adding one anyway adds nav overhead with no benefit. Splitting is a tool, not a virtue.

## Generic References Bucket

Purpose-shaped material should not be wrapped in `References/` just because the example structure shows that folder. If the domain naturally has `Operations/`, `Project_Onboarding/`, `Providers/`, `Templates/`, or `Schemas/`, those folders belong at the skill root. `References/` is for miscellaneous supporting material that lacks a clearer purpose.

## Murky labels

A perfectly organized file tree with vague link descriptions ("see advanced.md for more") fails. The agent cannot decide whether to enter, so it either skips relevant material or loads everything to be safe. The labels are load-bearing. Bad labels make a split-up skill worse than a flat one.

## Dump-into-references

If a reference file is itself a dump of every related fact, the pattern was pushed down a level rather than applied. A reference file should be its own navigation-first doc: orientation, common case, links if it needs them. The recursion is the point.

## Marketing prose

Skill files do not need to sell the skill. The agent has already loaded it. Cut sentences that describe how powerful, flexible, or important the skill is. Replace with what it does and how it is used.
