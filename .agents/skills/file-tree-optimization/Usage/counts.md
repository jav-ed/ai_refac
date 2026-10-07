# File and folder counts

Use counts to locate folders that need a responsibility review and to compare a proposed grouping with the original. Distinguish recognized source files, all visible regular files, and direct versus recursive counts before drawing a conclusion.

## Native eza source summary

eza 0.23.5 has no general file-and-folder count summary. Its code summary reports recognized source-file counts and lines by language:

```bash
eza --code --color=never -- src
```

That is useful for understanding a source area's size and languages, but excludes unrecognized file types and does not report folder counts. Its scan is recursive; do not interpret it as the direct-file count used to investigate a crowded folder.

## Exact structural counts

For exact structural counts, run the bundled counter from the repository root, replacing `src` with the relevant directory:

```bash
python3 .agents/skills/file-tree-optimization/Code/count_Tree.py src --recursive
```

Omit `--recursive` for direct children only. The helper requires Python 3 and, by default, a Git working tree. The output reports:

- `totals`: files, descendant folders, symlinks, and other filesystem entries. The inspected root is not itself counted as a folder. Without `--recursive`, these totals describe only its direct children.
- `directories`: each inspected folder's path and direct counts. With recursion, these identify crowded folders and clusters of single-file folders without confusing recursive totals with direct counts.
- Filter settings: hidden entries are omitted unless `--all` is supplied; `.git` metadata is always excluded. Git ignore patterns apply to entries, including tracked paths that match those patterns, using `git check-ignore --no-index`. Use `--include-ignored` for explicit unfiltered counts or for a directory outside Git.
- Symlinks are counted separately and their targets are not traversed. Empty visible folders count too. No files are moved or renamed, and counts do not decide the correct grouping.

The helper uses filesystem types and NUL-delimited Git paths, so spaces and newlines in names do not distort the count. Filesystem or Git failures exit nonzero with an explicit error and no partial JSON result. Match its hidden/ignored scope to the eza view before comparing them. Resolve the script from this skill's location if it is installed outside `.agents/skills`.

Avoid deriving structural counts by parsing eza's rendered tree. In 0.23.5, combining `--only-files` and `--no-symlinks` also returned directories and links in a verified fixture, although each flag worked individually. The companion counter makes entry types and filtering explicit.

## Interpret the result

Use each `directories` row for the inspected folder's direct count, including colocated tests. Recursive `totals` describe the whole scanned subtree; they cannot establish how crowded an individual folder is. Inspect both parent and children, and make generated or vendored exclusions explicit.

Counts direct attention; they do not determine ownership or justify a move by themselves. Read the grouping rules before treating a high file count or several single-file folders as a structural problem.

- [Grouping rules and examples](../Logic/grouping.md): the more-than-nine-file signal, single-file folders, purposeful subdivisions, and relocation to a better owner.
- [eza views and filtering](eza.md): inspect the same scope visually, reveal excluded entries, and check installed option support.
- [Counter implementation](../Code/count_Tree.py): the filesystem and Git logic behind the JSON output.
- [Everyday commands](../SKILL.md): begin with a scoped tree and a direct-count command.
