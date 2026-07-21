# doc_Start

*This `doc_Start.md` is the docs entry point, structured so an agent can quickly decide what to read and what to skip. It opens with a short summary of the repo and key entry-point files, then routes to each topic area through labeled links. Open a linker only when the task calls for it; the labels are written to make that decision possible without clicking.*

A short summary of the repo: what it is, what it does, and the broad shape (kind of system, primary stack, what is in scope and what is out of scope). One short paragraph is usually enough, long enough that an agent dropping in cold knows the lay of the land before scanning the link list below.

Entry point(s): the key files an agent would need to know to orient in the code without reading anything (e.g. `src/main.js`, `src/Gen/generator.js`). Keep it to one line per file.

## Docs

- [Area name](Area/linker_Area.md): describe what kinds of tasks or questions belong here, specific enough that an agent can decide without clicking
- [Area name](Area/linker_Area.md): same
- [Specific doc if area is small](Area/doc.md): same

## Repo References

- [Internal repo paths](Setup/internal_Repo_Paths.md): first-party repo shortcut names and host-scoped checkout paths. Use this for repos the project owns or operates across machines.
- [External reference repos](Setup/repos_List.md): third-party or external shallow clones that belong under the gitignored `/Repos/` folder. Use this for upstream source/docs clones used as references.

Note: this file lives at `Project_Manag/Docs/doc_Start.md`, so all link paths above are relative to `Project_Manag/Docs/`. Do not prepend `Project_Manag/Docs/` to them.

The italicized paragraph at the top is the navigation-first preamble. Copy it verbatim into every `doc_Start.md` (rule 1 in `writing_Rules.md`); only the summary, entry points, and link list change per repo.
