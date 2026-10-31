# Shipping skills with skillmirror

`skillmirror` is the CLI that moves a skill between the vault and every project that installs it. Files are copied, never symlinked, so a project stays git-tracked and the skills work over SSH. This page is the command reference; [Shipping a change](../SKILL.md#shipping-a-change) in SKILL.md is the order to follow and why a complete replace matters.

## Updating an existing skill

```bash
skillmirror promote coding --dry-run   # preview what would replace the vault's copy
skillmirror promote coding             # replace it (kept as a backup, staged in git, never committed)
skillmirror promote coding --sync      # replace it, then ask to sync every other project too
skillmirror sync coding --dry-run      # on its own: preview the push to every project that has it
skillmirror sync coding -y             # push without asking
```

`promote` takes the project's current `.agents/skills/<name>` and makes it the vault's. It never merges: the vault folder is removed first, so a page the edit deleted does not linger behind a partial copy. `--all` promotes every skill the project has changed, in one pass. Without `--project`, it uses the project around the current folder.

`sync` only ever updates a skill a project *already has*; it will not install one somewhere new. Scope it with a skill name, `--group <vault-folder>`, or `--profile <name>` from `<vault>/config.yaml`; `--project` limits it to one project. `--check` writes nothing and exits 1 when something differs, for a CI-style gate.

## Bringing a skill into the vault for the first time

```bash
skillmirror new my-skill --group web --description "One sentence. Use when ..."
skillmirror adopt my-skill --from /path/to/project --group web
```

`new` scaffolds a skill from a template directly in the vault (staged in git, never committed): write it there, or draft it in a project first and bring it in with `adopt`, which copies a project's existing `.agents/skills/<name>` into the vault (also staged, never committed) the first time a skill is good enough to share. `--group` is the vault folder to file it under (`web`, `web/seo`, …); it is created if missing.

## Starting a new project

```bash
skillmirror init /path/to/new-project coding search doc-start
skillmirror init /path/to/new-project --profile backend
```

Creates the directory (it must not exist or be empty), installs the vault's mandatory skills plus whichever names, `--group`, or `--profile` is given, and by default makes a git repo, runs the `doc-start` skill's scaffold script, and writes an `AGENTS.md`. `--no-git`, `--no-scaffold`, `--no-mandatory`, and `--no-agents-md` turn off any one of those.

## Looking before changing anything

- `skillmirror status` / `skillmirror diff`: how every project stands against the vault, or the exact lines a sync would bring in and take away. Both write nothing; both exit 1 when something differs.
- `skillmirror changes`: which of a project's skills are edited relative to the vault, behind it, or missing from it entirely. Run this before `promote` to confirm what is actually about to move.
- `skillmirror info <skill>`: where a skill lives in the vault, its files, which profiles name it, and what it looks like in each project.
- `skillmirror doctor`: checks the vault, the configuration, and the machine; exits 1 for warnings, 3 for errors.

## Undoing a run

Every `sync`, `promote`, `adopt`, `new`, `push`, or `init` is a recorded run. `skillmirror history` lists them, newest first; `skillmirror undo` reverts the most recent one (undoing is itself a run, so a second `undo` redoes it). Prefer this over hand-editing files back when a sync or promote turns out wrong.

## Mandatory skills across every project

```bash
skillmirror push --dry-run
skillmirror mandatory
```

`push` installs the vault's mandatory skills into every project that has a skills directory; it is how a newly-added mandatory skill reaches projects that predate it, separate from `sync`'s per-skill updates. `skillmirror mandatory` shows or changes which skills `push` installs everywhere.
