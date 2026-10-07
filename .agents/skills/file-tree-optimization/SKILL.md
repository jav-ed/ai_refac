---
name: file-tree-optimization
description: "Uses eza to inspect repository trees and improve their organization by responsibility, ownership, and meaningful naming. Use when reviewing or restructuring a file tree, consolidating repeated prefixes, reducing folder clutter, or planning where new code belongs."
---

# File-tree optimization

Purposeful grouping gives developers an easy time finding and understanding related things. Organize by responsibility, with meaningful names and predictable homes for related implementation, rules, tests, and supporting material. Use `eza` to see the current tree, then inspect the source: a tree alone cannot establish ownership or dependencies.

## Everyday commands

Assume `eza` is installed. Read the repository's instructions and documentation entry point, check its working-tree state, then choose the view needed. Run these from the repository root; replace `src` with the relevant directory.

**1. Overview of the repository, three levels deep:**

```bash
eza --tree --level=3 --git-ignore --group-directories-first \
  --color=never --icons=never --hyperlink=never -- .
```

**2. Inspect a specific subtree, three levels from that directory:**

```bash
eza --tree --level=3 --git-ignore --group-directories-first \
  --color=never --icons=never --hyperlink=never -- src
```

**3. Inspect a folder's direct contents, including hidden entries:**

```bash
eza --tree --level=1 --all --git-ignore --ignore-glob='.git' \
  --group-directories-first --color=never --icons=never --hyperlink=never -- src
```

**4. Include hidden configuration and tool folders in the overview:**

```bash
eza --tree --level=3 --all --git-ignore --ignore-glob='.git' \
  --group-directories-first --color=never --icons=never --hyperlink=never -- .
```

**5. Count direct files and folders in a directory:**

```bash
python3 .agents/skills/file-tree-optimization/Code/count_Tree.py src
```

The counter reports files, folders, and symlinks separately as JSON. Add `--recursive` for subtree totals and each folder's direct counts. Resolve the helper relative to this skill if installed elsewhere.

Git filtering follows ignore rules; it does not inherently recognize build output. `--all` reveals hidden entries while keeping Git filtering. Always pass an explicit target, including `.`. Depth 3 is a viewing window: drill into relevant subtrees. An empty result or failed command needs investigation before it supports a structural conclusion.

## Decision and execution checklist

1. **Inspect relationships.** Read entry files and use targeted `rg` searches for callers and dependencies. Review repeated names, more than nine direct files, single-file folders, scattered subjects, and inaccessible shared core. Use the logic guides below for the rules and examples.
2. **Choose a purposeful home.** Identify the owner and what its children share; that home may be elsewhere in the repository. Show the relevant before/after tree, old-to-new paths, and how related work becomes easier to find. Preserve package and runtime contracts; check destination collisions. Leave a clear structure unchanged when evidence supports it.
3. **Execute the authorized scope.** An audit produces findings; an implementation request includes moves, reference updates, and verification. Use the Refac skill for supported moves and renames, following its language-specific operation, package-root, and batch rules. Report unsupported operations or failures before choosing another method. Preserve existing work and comments; moving a file does not require renaming its public symbols.
4. **Verify the result.** Audit imports, dynamic imports, discovery paths, configuration, tests, and docs, including references outside Refac's coverage. Regenerate derived files through their owning tools. Check a fresh tree, direct counts, stale paths, and affected behavior. Report what passed and any remaining limitations.

## Read when needed

- [Grouping and names](Logic/grouping.md): repeated prefixes, crowded and single-file folders, meaningful local names, and useful nesting, with before/after examples.
- [Ownership and boundaries](Logic/ownership.md): complete subject grouping, accessible shared core, lifecycle distinctions, cohesive extraction, and aligned supporting content.
- [Detailed eza usage](Usage/eza.md): filtering, directory views, metadata, symlinks, Markdown snapshots, and option compatibility.
- [File and folder counts](Usage/counts.md): native source summaries versus exact companion counts, JSON fields, filtering options, and direct versus recursive totals.
- [Setup](Setup/installation.md): install a missing eza, follow the machine's tool policy, and verify executable resolution and Git support.
- [Coding conventions](../coding/SKILL.md): established naming, one responsibility per file, file-length limits, and comment rules.
- [Refac CLI](../refac-cli/SKILL.md): perform supported moves with reference updates using the right language operation, package root, and batch size.
