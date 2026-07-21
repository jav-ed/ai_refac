# Process

When asked to create or improve repo docs:

1. Identify the repo gist.
2. Survey which doc areas exist and what they contain.
3. For large areas, check if a `linker_<Topic>.md` exists. If not, create one.
4. Write or refine `doc_Start.md`. Keep it minimal, link to linker files or directly to docs.
5. Remove duplicated explanations. One file owns a topic; others link to it.

## When editing an existing linker

A linker rots fast: docs get added, renamed, or moved, and the linker stays the same. Before editing or trusting a linker, verify it.

1. Glob the folder it sits in to see what `.md` files actually exist.
2. Check every entry in the linker still points to a file that exists.
3. Check every sibling doc is listed in the linker (no missing entries).
4. If a sub-folder has its own linker, the parent should point to the sub-linker, not enumerate the leaves.
5. Spot-check that descriptions still match the H1 or intro of the linked doc. If a doc's content has drifted from its description, update the description.
6. While you're there, glance at the H1s of the leaf docs in the folder. If most siblings follow a pattern (`# Topic: Subtopic`) but one is an outlier, fix the outlier to match.
