---
name: tmux
description: This skill should be used when the user needs to "run a command in tmux", "use a persistent terminal", "execute sudo commands", "interact with a TUI application", "capture terminal output", or "run privileged commands with password entry".
version: 0.1.0
---

# tmux Terminal Interaction

Use native tmux commands to interact with terminal sessions. Enables persistent terminal access, privileged command execution, and reliable output capture from a real TTY.

## References

- [Output capture](references/output_Capture.md) — which capture method to use and when (file+sentinel, capture-pane, what to avoid)
- [sudo workflow](references/sudo_Workflow.md) — how to run privileged commands when a password is required
- [Session & pane management](references/session_Pane.md) — creating, splitting, and killing sessions and panes

## Standard Workflow

```
1. Get pane ID once  → tmux list-panes -a -F "#{session_name}:#{pane_id}"
2. Clear outputs     → rm -f /tmp/tmux_out.txt /tmp/tmux_done.txt
3. Execute           → tmux send-keys -t '%X' 'cmd > /tmp/tmux_out.txt 2>&1; echo ok > /tmp/tmux_done.txt' Enter
4. Poll              → for i in $(seq 1 20); do [ -f /tmp/tmux_done.txt ] && break; sleep 0.5; done
5. Read result       → cat /tmp/tmux_out.txt
```

Reuse the same pane ID throughout the session — IDs are stable for the lifetime of the pane.

## Quick Reference

| Task | Command |
|------|---------|
| List sessions | `tmux list-sessions` |
| List panes | `tmux list-panes -a -F "#{session_name}:#{pane_id}"` |
| Send command | `tmux send-keys -t '%X' 'cmd' Enter` |
| Capture (short output) | `tmux capture-pane -p -t '%X' -S -10` |
| Capture (reliable) | file + sentinel — see [output_Capture.md](references/output_Capture.md) |
| Attach (for sudo) | `tmux attach -t name` |
| Create session | `tmux new-session -d -s name` |
| Kill session | `tmux kill-session -t name` |
