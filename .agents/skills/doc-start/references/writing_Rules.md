# Writing Rules

1. Start with the gist of the repo.
   - One or two sentences max. What it is, what it does.
2. Keep `doc_Start.md` and all linker files shallow in structure, but not in label text.
   - They route the reader. They do not explain.
   - Link labels must be rich enough that the reader can make a routing decision without clicking. A bare area name ("Architecture") is not enough. Say what kinds of questions or tasks belong there.
3. Prefer progressive disclosure.
   - Short summary first, link second, deep detail in the target file.
4. Do not make recommendations.
   - No reading order, no guided path, no "start here first" unless explicitly asked.
5. Create `linker_<Topic>.md` for large areas.
   - If Architecture, Investigation, or any area grows large, add a linker file and link to it from `doc_Start.md`.
6. Avoid duplication.
   - Shared content belongs in one file. Link to it everywhere else.
7. Do not dump every doc into `doc_Start.md`.
   - Only include what helps the reader decide where to go next.
8. Use link labels that explain why to click.
   - Bad: `podman.md`
   - Better: `Podman container standards`
9. Keep agency with the reader.
   - The goal is to avoid loading the agent with context it did not ask for.
