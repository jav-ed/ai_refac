# Agent integration

Read this file only to make an agent harness find this skill. The skill is the folder `refac-cli/`; its entry point is `SKILL.md`. Put the folder where the harness looks for skills: a symlink when the folder lives in a repository (one source of truth), a copy otherwise.

## Claude Code

Claude Code reads project skills from `.claude/skills/`. When the skills are kept under `.agents/skills/`:

```bash
# from the repo root
mkdir -p .claude
ln -s ../.agents/skills .claude/skills
```

Add `.claude/skills` to `.gitignore`: the content is already tracked under `.agents/skills/`, so committing the symlink would duplicate it. Once the symlink is in place Claude Code picks up every skill there; no further configuration is needed.

For one user across all projects, copy or symlink `refac-cli/` into `~/.claude/skills/` instead.

## Other agent harnesses

The same pattern applies to any harness that resolves skills from a local directory: point it at `refac-cli/` (or at the folder that holds all skills) and it finds `SKILL.md` as the entry point.
