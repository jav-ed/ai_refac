# Gitignore

What a repository's `.gitignore` holds for the doc-start folders and for the agent tools that work in the repository, and where the lines go. Open it when bootstrapping a repo, when a tool leaves a new folder in `git status`, or when a tool downloads a program into the repo.

## The lines

| Line | Why it is ignored |
|---|---|
| `Repos/` | Third-party clones. The list of what belongs there is tracked in `Project_Manag/Docs/Setup/repos_List.md` ([Repos convention](repos_Convention.md)). |
| `Scratch/` | Disposable agent output ([Scratch cleanup](scratch_Cleanup.md)). |
| `.impeccable/` | The working files of the impeccable design skill: surface briefs, mock-ups, review and question notes and a local config. Per machine and per worktree, not source. |
| `.playwright*` | What `playwright-cli` writes into the folder it runs in (`.playwright-cli/`: console logs and page snapshots). The star also covers any other Playwright tool folder that starts the same way. |
| `**/skills/impeccable/scripts/bin/` | The program the impeccable skill downloads on its first run into its own skill folder (about 20 MB for one platform). A download is never part of history. |

`Code/bootstrap.sh` adds each line if it is missing (exact-line match, so a repo that already has the line gets no duplicate). A repo that did not bootstrap adds them by hand.

## Where the lines go

- Together, with the other doc-start and agent-tooling lines, not appended at the end of the file. A line pasted after a "keep last" block or a safety net sits outside the sections a reader checks.
- Git uses the last matching rule, so a re-include (`!pattern`) goes below the rule it overrides.
- No blanket rules for a tool's sake (`*.json`, `*.bin`, `bin/`): they hide tracked files in repos that need them.

## Ignoring does not untrack

A file that was committed before its line was added stays tracked and keeps showing changes. Stop tracking it with `git rm --cached <path>` (the file stays on disk), then commit. A big file that is already in history needs more than an ignore line: the history has to be rewritten, which is the owner's decision, so report it instead of doing it.

After the first run of any tool that downloads a program or writes a new dot-folder, look at `git status` once and ignore what it left.
