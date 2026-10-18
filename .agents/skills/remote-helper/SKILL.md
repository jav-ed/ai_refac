---
name: remote-helper
description: "Tells the agent which of two machines it is running on, the local dev machine `javPc` or the production server `mail.zetunweb.com` (informally g12), and how to run a command on the other one over SSH. Use when about to run host-specific commands (systemctl, journalctl, secrets edits, deploys, local dev servers), when a task needs something done on the other machine, or when the user mentions javPc, g12 or running commands on one machine from the other."
---

# remote-helper

Two machines work together: `javPc`, the local dev machine, and g12, the production server (hostname `mail.zetunweb.com`). Many commands only make sense on one of them, and some tasks need work on the other one.

## Detect the current machine

```bash
hostname
```

- `javPc`: the local dev machine.
- `mail.zetunweb.com`: the production server, informally called g12.
- anything else: stop and ask the user before running host-specific commands.

Run it once at the start of a task with host-specific commands (`systemctl`, `journalctl`, service files, secrets under `/etc` for server work; dev servers, browsers, GUI tools for local work). Purely repository-local work does not depend on the host.

## Run something on the other machine

Both directions use a ready SSH alias, key-only, without prompts.

| From | To | Command | Connection |
|---|---|---|---|
| g12 | javPc | `ssh javpc "<command>"` | private NetBird network; javPc accepts only g12; restarting NetBird on either machine cuts this path |
| javPc | g12 | `ssh g12 "<command>"` | public SSH, key only; stays up while NetBird restarts |

Test reachability with `ssh javpc hostname` (from g12) or `ssh g12 hostname` (from javPc); it must print the other machine's name.

Syntax:

- One simple command: `ssh <alias> "<command>"`.
- Several steps, patterns, or tools from the login PATH (`himalaya`, `fnox`): `ssh <alias> "bash -lc '<steps>'"`. The login shell on javPc is zsh and fails on patterns that match nothing; `bash -l` also gives the right PATH.
- In scripts and loops use `ssh -n`, otherwise ssh swallows the script's standard input.

Rules:

- Say which machine each command runs on when you report it.
- Repository checkouts live in different folders on the two machines; the current project's host-scoped path document (`Project_Manag/Docs/Setup/internal_Repo_Paths.md` when it has one) says where.
- Sudo: both machines need the user's password. Test with `sudo -n true`; if it says a password is required, ask the user to run `sudo -v` on that machine. Never guess passwords.

## When it fails

- **javPc must be on and connected.** If `ssh javpc hostname` times out, report it; do not retry in a loop.
- **Missing alias or a question from ssh** (host key, password prompt): stop and tell the user.
