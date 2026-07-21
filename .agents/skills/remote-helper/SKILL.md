---
name: remote-helper
description: Detect whether the current shell is running on the local dev machine (`javPc`) or on the remote production server (`mail.zetunweb.com`, informally "g12"). Use when the agent is about to run host-specific commands (setup, deploy, journalctl, secrets edits, local `go run`) and needs to confirm which machine it is on before proceeding.
---

# remote-helper

Many commands in this project only make sense on one of the two machines. Production install, `systemctl`, `/etc/contact-form/secrets.env`, and `journalctl -u contact-form` only run on the server. `go run ./cmd/server`, curl against `127.0.0.1:8080`, and edits inside `07_Customers` only make sense locally. Running the wrong one on the wrong host wastes time at best and breaks the live service at worst. Before any host-specific command, confirm where you are.

## Detect the current machine

```bash
hostname
```

Match the output:

- `javPc` — local dev machine.
- `mail.zetunweb.com` — the production server, informally called "g12".
- anything else — neither dev nor prod. Stop and ask the user before running host-specific commands.

That single command is the whole check. There is no fancier signal needed: the hostnames are stable and unambiguous.

## When to run the check

Run it once at the start of any task that includes commands like:

- `systemctl`, `journalctl`, edits under `/opt/zetun/` or `/etc/contact-form/` — server-only.
- `go run ./cmd/server`, `curl 127.0.0.1:8080`, edits inside the `07_Customers` working copy — local-only.
- `ssh g12 ...` — only makes sense from local; running it on the server is a no-op or worse.

If the task is purely repo-local (reading code, editing source files, running `go build` or `go test`), the host does not matter and the check can be skipped.

## Cross-references

- [Setup guide](../../../Project_Manag/Docs/Setup/setup_Guide.md): the production install/deploy workflow that assumes `hostname` returns `mail.zetunweb.com`.
- [Local testing](../../../Project_Manag/Docs/Setup/local_Testing.md): the local dev workflow that assumes `hostname` returns `javPc`, including the SSH tunnel fallback if port 587 is blocked.
