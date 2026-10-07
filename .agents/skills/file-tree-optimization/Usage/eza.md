# Detailed eza inspection

Use these views when the everyday tree leaves a relevant question unanswered. eza displays filesystem structure and metadata; source inspection establishes responsibilities. Commands below use `src` as the example scope. Replace it with the directory being investigated.

The explicit color, icon, and hyperlink controls keep output suitable for agents and Markdown. `--` ends option parsing before the target path.

## Understand what a view excludes

`--git-ignore` filters Git-ignored paths, including applicable nested ignore rules. It does not mean tracked-files-only: untracked files that are not ignored still appear. `--all` reveals hidden names but does not cancel Git filtering. Build output disappears only when the active rules exclude it.

For a relevant ignored or otherwise filtered subtree, inspect it explicitly without Git filtering:

```bash
eza --tree --level=2 --all --ignore-glob='.git' \
  --group-directories-first --color=never --icons=never --hyperlink=never -- src
```

For large non-Git trees, use explicit exclusions appropriate to the task. Quote pipe-separated globs so the shell does not interpret them:

```bash
eza --tree --level=3 --all --ignore-glob='.git|node_modules|target|dist' \
  --group-directories-first --color=never --icons=never --hyperlink=never -- .
```

Those are example exclusions, not universal names to hide. State material exclusions in the findings. Do not exclude a path merely to make the structure or file count look cleaner. Use `git check-ignore -v -- path` when the reason a path is ignored matters.

## Directory shape and direct contents

A directories-only view helps locate broad domains in a large tree:

```bash
eza --tree --level=4 --only-dirs --git-ignore \
  --color=never --icons=never --hyperlink=never -- src
```

Return to a file-inclusive view before making placement decisions. To inspect all direct children without recursing or applying Git filtering:

```bash
eza --oneline --all --group-directories-first \
  --color=never --icons=never --hyperlink=never -- src
```

Do not use a rendered tree as a machine-readable path inventory. Use the companion counter for exact structural counts; a truncated visual tree cannot establish them.

## Metadata and links

Use a long listing when Git status, permissions, size, or link targets explain an observed entry:

```bash
eza --long --header --git --all --git-ignore --group-directories-first \
  --color=never --icons=never --hyperlink=never -- src
```

`--git` adds status information; `--git-ignore` filters ignored entries. Neither replaces a repository status check before editing. Do not infer architectural importance from file size or modification time.

eza does not descend into symlinked directories by default. `--follow-symlinks` enables traversal; use it only after checking the target and that it belongs in the inspection scope. `--dereference` concerns displayed link information. Do not mistake a linked dependency checkout for locally owned source.

## Export a Markdown snapshot

Run this Bash block from the repository root. Set `tree_root` to the intended subtree and keep disposable output under Scratch. The temporary output is in the destination directory; a failed `eza` run exits before replacement, preserving any existing snapshot.

```bash
bash <<'BASH'
set -euo pipefail
tree_root='src'
tree_output='Scratch/Audit/file_Tree.md'
mkdir -p "$(dirname "$tree_output")"
tree_temp=$(mktemp "${tree_output}.tmp.XXXXXX")
trap 'rm -f -- "$tree_temp"' EXIT
{
  printf '# File tree\n\nRoot: `%s`; depth: 3. Hidden and Git-ignored entries omitted.\n\n```text\n' "$tree_root"
  eza --tree --level=3 --git-ignore --group-directories-first \
    --color=never --icons=never --hyperlink=never -- "$tree_root"
  printf '```\n'
} > "$tree_temp"
mv -- "$tree_temp" "$tree_output"
BASH
```

A requested committed snapshot belongs at the repository's documentation location and needs the corresponding scope/exclusion note. A snapshot is evidence of a particular inspection, not a live map or proof that imports still resolve.

## Option compatibility

Use `eza --help` for the installed build's supported flags and `eza --version` to identify it. The main commands were verified with `0.23.5` with Git support. If an option fails, report the failure and resolve the version or build mismatch explicitly; do not silently drop filtering and present a different scope as equivalent.

Pass an explicit target in automation. In 0.23.5, non-terminal stdin selects input-path reading when no path argument is present; empty stdin can yield no output with a successful exit status. `-- .` explicitly lists the current directory and avoids this ambiguity. An unexpectedly empty listing needs investigation before it can support a structural conclusion.

- [Versioned stdin handling](https://github.com/eza-community/eza/blob/v0.23.5/src/options/stdin.rs): the installed release's selection between command arguments and stdin paths.
- [Official manual](https://github.com/eza-community/eza/blob/main/man/eza.1.md): full option semantics and additional display modes. The main branch may describe features beyond the installed version.
- [File and folder counts](counts.md): native source summaries, exact companion counts, filtering, and direct versus recursive totals.
- [Setup](../Setup/installation.md): missing executable, incompatible build, and installation verification.
- [Grouping logic](../Logic/grouping.md): use inspection evidence to reason about meaningful grouping and naming.
