# Folder Guidance

The doc system lives under `Project_Manag/`, split into two top-level folders:

- `Project_Manag/Docs/`: reference material (Architecture, Decisions, etc.)
- `Project_Manag/Live_Working/`: active work and ongoing planning

Two more folders sit at the repo root: `/Repos/` for shallow clones of third-party repos, and `/Scratch/` for throwaway agent-generated outputs. Both are fully gitignored. See the sections below.

## Docs/ subfolders

| Folder | Required? | Use for |
|---|---|---|
| `Architecture` | required | Structure, systems, technical layout, integration boundaries |
| `Decisions` | required | Important decisions, tradeoffs, ADR-like notes |
| `Descr` | required | What the repo or product does, domain model, conceptual descriptions |
| `Research` | required | Quick lookups, online finds, lightweight external content gathered for context |
| `Setup` | required | Local setup, environment bootstrap, install steps. Also home to `internal_Repo_Paths.md` for first-party repo shortcuts and `repos_List.md` for external clones in gitignored `/Repos/` |
| `Brand` | optional | Brand assets, voice, visual identity |
| `Investigation` | optional | In-depth study of a tool, system, or topic. Sustained work that can grow its own subfolder structure |

The five required folders exist on every repo from bootstrap. Optional folders are added only when the project has material in that category.

`Research` vs `Investigation`: `Research` is for quick, lightweight external lookups (a few links or excerpts gathered for context). `Investigation` is for sustained, in-depth study of a single subject and can grow its own internal subfolder structure as the work develops.

## Live_Working/ contents

| File | Required? | Use for |
|---|---|---|
| `open_Issues.md` | required | Active items, in-progress work, things to track |

Add other files in `Live_Working/` as the project's active-work needs grow (task pools, sprint plans, etc.).

## Internal repo paths

`Project_Manag/Docs/Setup/internal_Repo_Paths.md` records first-party repo shortcut names and host-scoped full checkout paths. Use it for repos we own or operate across machines, such as `Installations` or `10_Web_Runtime`. Docs should use the shortcut name in prose and link to this file when an absolute path matters.

Before relying on a full path from this file, use the repo's host-detection convention if one exists. In repos that use `remote-helper`, run `hostname` and match the observed host before using host-specific commands.

## Repos/ at the repo root

`/Repos/` lives at the repo root (sibling of `Project_Manag/`), not under `Docs/`. It holds shallow `git clone --depth 1` clones of external or third-party repos used as source-code or documentation reference. The folder is fully gitignored, so the clones never enter git history. The manifest describing what should be present and how to re-clone it lives separately, under git, at `Project_Manag/Docs/Setup/repos_List.md`. First-party repos do not belong in `/Repos/`; track them in `internal_Repo_Paths.md`. For the full convention, see `repos_Convention.md`.

## Scratch/ at the repo root

`/Scratch/` lives at the repo root (sibling of `Project_Manag/`) and is the standard location for throwaway, agent-generated outputs. It always has three subfolders, used for the categories below:

| Subfolder | Use for |
|---|---|
| `Scratch/Screenshots/` | Playwright (or other) screenshots of the running app |
| `Scratch/Design/` | Design references and mockups pulled in for visual comparison |
| `Scratch/Audit/` | Audit reports and snapshots (accessibility, performance, etc.) |

The whole folder is gitignored end-to-end. Nothing inside is ever committed; if a file in `Scratch/` matters, copy it elsewhere first. `bootstrap.sh` creates the folder and its three subfolders unconditionally — projects that do not use them simply leave them empty.
