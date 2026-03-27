# Agent Integration

The skill lives in `.agents/skills/refac-cli/` inside the repo. To wire it into an AI agent harness, symlink that folder rather than copying — this keeps a single source of truth.

## Claude Code

Claude Code looks for skills in `.claude/skills/` at the project root.

```bash
# from the repo root
mkdir -p .claude
ln -s ../.agents/skills .claude/skills
```

`.claude/` should be committed to git. The symlink target (`.claude/skills`) should be gitignored — the content is already tracked under `.agents/skills/`, so committing the symlink would duplicate it.

Add to `.gitignore`:

```
.claude/skills
```

Once the symlink is in place, Claude Code picks up all skills in `.agents/skills/` automatically. No further configuration needed.

## Other agent harnesses

The same symlink pattern applies to any harness that resolves skills from a local directory. Point it at `.agents/skills/refac-cli/` (or `.agents/skills/` for all skills) and the harness will find `SKILL.md` as the entry point.
