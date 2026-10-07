# Scratch Cleanup

Every doc-start scaffold provides `just scratch-clean`. Scratch is disposable: the command removes all contents, then recreates empty `Agent_Tasks/`, `Audit/`, `Design/`, and `Screenshots/` folders. It has no retention window or file-type exceptions.

## Installation

The [bootstrap script](../Code/bootstrap.sh) installs the command when explicitly invoked. To add only the cleanup tooling to an existing repository, run `bash <skill-path>/Code/install_Scratch.sh <repo-root>` during an authorized setup change. This installs a Bash helper at `Code/Development/Scratch/clean.sh` and adds a Just recipe; Bun and Node are not required.

Existing `scratch-clean` recipes, including imported recipes, are retained. Verify that an existing implementation follows the same contract. Other Just recipes and their default behavior stay intact. A new Justfile defaults to listing commands. Conflicting helper files and ambiguous Justfiles cause descriptive errors instead of being overwritten.

Before making an existing Scratch directory disposable, inspect its callers. Move operational dependencies to their owners: reusable scripts into tracked code, deployment artifacts into a dedicated ignored directory such as `Cache/CI/Artifacts/`, and lasting conclusions into docs. Update callers and stored paths together. Do not create duplicate cleanup guides when the project already has one.

## Cleanup contract

- Delete every Scratch entry, including hidden files, task notes, logs, images, JSON, source copies, and extra directories. Preserve the Scratch root and recreate only the four standard folders.
- Resolve the project from the installed helper's location. Accept no user-supplied deletion path.
- Refuse a symlink or non-directory at the Scratch root. Remove nested symlinks themselves; never follow their targets or modify shared hard-link bytes.
- Run only on explicit invocation, after work using those files has finished. Installing or reading this skill never authorizes a wipe.
- Keep required runtime inputs and CI artifacts outside Scratch. Durable docs must remain understandable after it is emptied.

## Verification

Run `python3 <skill-path>/Code/verify_Scratch.py` to exercise bootstrap and the real Just command in temporary repositories. It checks repeatable setup, existing and imported recipes, the safe default command, hidden files, copied source, symlinks, hard links, and preservation of files outside Scratch. Test fixtures never use a working repository's Scratch folder.
