# When to add code

Skills can bundle code in a `Code/` subfolder when the skill needs deterministic helpers. Generated code is fine for most cases.

## Add code when

- The operation is deterministic: validation, formatting, parsing, structured transforms.
- The same logic would be regenerated repeatedly across uses of the skill.
- Errors need explicit, consistent handling that prompt-generated code drifts on.
- The operation has tricky edge cases the agent should not re-derive each time.

Bundled code reduces variance and keeps the agent focused on the task instead of re-deriving the same logic each invocation.

## Skip code when

- The work is one-off or context-dependent.
- The agent is better served by writing the code itself with the user's local conventions.
- Maintaining the bundled code would lag the language model's natural ability.

## Conventions

- Put code in a `Code/` subfolder at the skill root.
- Name files by what they do, not by what they wrap (`validate_frontmatter.py`, not `helper_2.py`).
- Document each piece in SKILL.md or a reference file with one-line purpose and invocation.
- Treat bundled code as part of the skill's contract: changes to its behavior are changes to the skill.
