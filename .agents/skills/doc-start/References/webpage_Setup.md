# Webpage Setup

Convention that applies **only to webpage projects** (anything that runs a dev server in the browser: Astro, Next.js, plain Vite, static HTML served by a tool, etc.). Non-webpage projects skip this entirely.

## What gets added

A single file: `Project_Manag/Docs/Setup/visual_Testing.md`. It documents how the agent runs visual tests against the project's dev server, what to install, what to check before screenshotting, and where to put outputs. The file is created on demand, not by `bootstrap.sh` — the bootstrap script cannot reliably tell whether a project is a webpage project, so it leaves this to the agent.

## Why a separate file

Visual testing instructions are highly project-specific (port numbers, dev-server command, framework defaults) and they sit on the boundary between docs and runbook. Putting them in `Setup/` keeps them next to `repos_List.md` and `linker_Setup.md`, which already cover the "how do I bring this project up" axis. Embedding the same text in `linker_Setup.md` would mix navigation with operational instructions and make the linker noisy.

## Required content of `visual_Testing.md`

The file must cover four things, in this order:

1. **Skill check + self-install.** Before anything else, confirm the `playwright-cli` skill is available in this project. If it is not on `PATH`, the agent runs from the project root:

       playwright-cli install --skills

   This is the canonical install step. The file should state it explicitly so a fresh agent does not have to guess.

2. **Dev server check.** The actual port and dev-server command vary by stack. The Astro default is port 4321. The check is:

       ss -tlnp | grep <port>

   Any output means the server is up; empty output means start it (e.g. `bun run dev`, `npm run dev`, etc.). Specify the project's actual command, not a generic placeholder.

3. **Common commands.** At minimum, `playwright-cli open <url>` and `playwright-cli screenshot --filename=Scratch/Screenshots/<page>.png`. Always route output to `Scratch/Screenshots/`, never to anywhere tracked by git. Point to the `playwright-cli` skill itself for resize, multi-tab, tracing, and the long tail.

4. **`Scratch/` subfolder usage.** Restate the three-subfolder layout (`Screenshots/`, `Design/`, `Audit/`) so agents working on visual tasks do not have to look it up elsewhere. Note that the whole `Scratch/` folder is gitignored.

## Skill-check intent

The agent should not assume `playwright-cli` is installed just because a previous session used it. Check first, install if needed, then proceed. Failing the check and asking the user is acceptable; running screenshot commands that error out without explanation is not.

## What to avoid

- Do not reproduce the full `playwright-cli` skill reference inside `visual_Testing.md`. Link to the skill and stop. The local file's job is to capture what is project-specific (port, dev command, scratch routing), not to mirror the skill.
- Do not commit anything from `Scratch/`. The folder is gitignored on purpose; if a screenshot matters, copy it to a tracked location first.
- Do not put visual testing instructions anywhere other than `Setup/visual_Testing.md`. A second copy in `linker_Setup.md` or `doc_Start.md` will drift.
