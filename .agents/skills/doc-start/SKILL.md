---
name: doc-start
description: Write or reorganize repository documentation using a navigation-first structure. `Project_Manag/Docs/doc_Start.md` is the docs entry point. Large topic areas get their own `linker_Topic.md` file. Every file opens with a summary, then routes through clearly-labeled links.
---

# doc-start

Write docs so an agent can orient itself without being forced to read everything. The docs system is a map: `Project_Manag/Docs/doc_Start.md` gives the agent enough overview to decide what is relevant, and each linker file does the same for its sub-area. The agent navigates the system, it does not consume it. This skill follows the same pattern, applied to its own SKILL.md.

## The principle: navigation-first, not dump-first

An agent works best when it can quickly tell what in its context is relevant to the current task and what is not. Every irrelevant doc loaded competes with the relevant ones and degrades the agent's decisions. So the docs system must give the agent an easy time deciding what to read and what to skip, and what is not loaded must still be findable through good labels. The system must:

1. Give enough orientation upfront, at every level, that the agent understands the area.
2. Inline a summary at the top of `doc_Start.md` and every linker file, so the most-used context comes for free.
3. Route the long tail through clearly-labeled links, so the agent can decide what to load without opening the file.

The load-bearing piece is the labels, not the file split. A flat doc with sharp labels beats a deeply nested doc with murky ones every time.

## Doc system structure

Three tiers, top to leaf:

```
repo-root/
├── Repos/                                # third-party clones, fully gitignored
├── Scratch/                              # throwaway outputs, fully gitignored
│   ├── Screenshots/                      # playwright screenshots
│   ├── Design/                           # design references, mockups
│   └── Audit/                            # audit reports, snapshots
└── Project_Manag/
    ├── Docs/
    │   ├── doc_Start.md                  # docs entry point: summary + routing
    │   ├── Architecture/                 # required
    │   │   ├── linker_Architecture.md    # area summary + links to leaves
    │   │   ├── leaf_doc_a.md
    │   │   └── leaf_doc_b.md
    │   ├── Decisions/                    # required
    │   ├── Descr/                        # required
    │   ├── Research/                     # required
    │   ├── Setup/                        # required
    │   │   ├── linker_Setup.md
    │   │   ├── internal_Repo_Paths.md    # first-party repo shortcuts + host-scoped paths
    │   │   ├── repos_List.md             # external/reference repos in /Repos + clone commands
    │   │   └── visual_Testing.md         # webpage projects only: playwright + scratch usage
    │   ├── Brand/                        # optional
    │   └── Investigation/                # optional
    └── Live_Working/
        └── open_Issues.md                # required
```

`doc_Start.md` lives at `Project_Manag/Docs/doc_Start.md`, never at the repo root. Topic-area docs live under `Project_Manag/Docs/<Area>/` as siblings of `doc_Start.md`. Each area gets its own `linker_<Area>.md` once it has docs. Sub-areas can have their own linkers, in which case the parent points to the sub-linker, not the sub-linker's leaves. Link paths in `doc_Start.md` are therefore relative to `Project_Manag/Docs/` (e.g. `Architecture/linker_Architecture.md`).

Repo references are split by ownership. First-party repos use stable shortcut names in prose, with host-scoped full checkout paths recorded in `Project_Manag/Docs/Setup/internal_Repo_Paths.md`. External or third-party reference repos are shallow clones under gitignored `Repos/`, with clone commands recorded in `Project_Manag/Docs/Setup/repos_List.md`. For the full convention, see [Repos convention](References/repos_Convention.md).

`Scratch/` also lives at the repo root and is the standard location for throwaway, agent-generated outputs (screenshots, design references, audit snapshots). It always has three subfolders — `Screenshots/`, `Design/`, `Audit/` — and is fully gitignored end-to-end. Nothing inside is ever committed; if a file in `Scratch/` matters, copy it elsewhere first.

For **webpage projects only**, `Project_Manag/Docs/Setup/visual_Testing.md` documents how the agent runs visual tests against the project's dev server (playwright commands, dev server check, scratch folder usage). Non-webpage projects do not create this file. For the convention, see [Webpage setup](References/webpage_Setup.md).

When starting a new repo, the required folders (`Architecture`, `Decisions`, `Descr`, `Research`, `Setup`), `Project_Manag/Docs/doc_Start.md`, `Project_Manag/Docs/Setup/internal_Repo_Paths.md`, `Project_Manag/Docs/Setup/repos_List.md`, and `Live_Working/open_Issues.md` need to be present. The user can run `Code/bootstrap.sh` to create the full scaffold (folders + linker stubs + `doc_Start.md` + `internal_Repo_Paths.md` + `repos_List.md` + `Repos/` + `Scratch/` + three Scratch subfolders + `.gitignore` entries + `open_Issues.md`); the agent can also create missing pieces by hand, but should only run the script when explicitly asked. `Brand/` and `Investigation/` are added only when the project actually has material in those categories. `visual_Testing.md` is added only on webpage projects. For what each folder is for, see [Folder guidance](References/folder_Guidance.md).

## Writing rules (common case)

These are the rules used in nearly every doc-start task. The full ruleset including style conventions is in [Writing rules](References/writing_Rules.md).

- `doc_Start.md` and every linker file open with a summary of the area before the link list. They are not pure indexes.
- Every link uses `[Short label](path.md): description rich enough for the reader to decide without clicking`. Use a colon, never an em-dash.
- Link descriptions are not limited to one line. One line when one line covers it, five lines when the topic genuinely needs five. Clarity for navigation, not brevity.
- When a sub-folder has its own linker, the parent links to that sub-linker, not its leaves. The parent points to the door, the sub-linker handles the room.
- Do not dump every doc into `doc_Start.md`. Include only what helps the reader decide where to go next.
- Do not impose a reading order ("start here first") unless the user explicitly asks for one.

## File and folder naming

`.md` files use lowercase first letter (`writing_Rules.md`, `linker_Architecture.md`). Folders use uppercase first letter (`Architecture/`, `Project_Manag/`). For the full naming convention, see the `coding` skill.

## References

- [Writing rules](References/writing_Rules.md): the full numbered ruleset including style conventions: no em-dashes, *inshallah* usage, H1 sibling consistency.
- [Folder guidance](References/folder_Guidance.md): the canonical top-level folders under `Project_Manag/Docs/` and what each is for, plus `Repos/` and `Scratch/` at the repo root.
- [Repos convention](References/repos_Convention.md): the rules for first-party repo shortcuts in `internal_Repo_Paths.md` and external/reference clones in `Repos/` with `repos_List.md`: host-scoped path records, `--depth 1` shallow clones, gitignore pattern, manifest format, when to add a repo, and `gh` recipes for finding URLs and discovering docs-only sibling repos.
- [Webpage setup](References/webpage_Setup.md): convention for webpage projects only. Defines `Project_Manag/Docs/Setup/visual_Testing.md` (playwright-cli usage, dev-server check, `Scratch/` subfolders for screenshots, design references, audits) and the `playwright-cli install --skills` self-install step.
- [Process](References/process.md): step-by-step process for creating new docs and for editing or verifying an existing linker.
- [doc_Start template](References/doc_Start_Template.md): scaffold for the repo entry point.
- [Linker template](References/linker_Template.md): scaffold for sub-area linker files.
- [Failure modes](References/failure_Modes.md): pure-index linkers, leaf enumeration, dumping into `doc_Start.md`, vague labels, stale linkers, duplicate content, and other ways the navigation-first pattern gets misapplied.
