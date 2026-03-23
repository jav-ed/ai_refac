---
name: tmux
description: Interact with tmux terminal sessions for shared terminal access, run interactive commands, execute sudo commands, and capture terminal output. Use when user needs terminal access, wants to run commands in a persistent terminal, needs to interact with TUI applications, or requires sudo with password entry.
---

# tmux Terminal Interaction

Use native tmux commands to interact with terminal sessions. This allows persistent terminal access, running interactive commands, and executing commands that require a terminal (like sudo).

## Key Use Cases

- **Shared terminal**: Access a terminal session that persists across conversations
- **Interactive commands**: Run commands that need user interaction
- **sudo commands**: Execute privileged commands, including password entry
- **TUI applications**: Interact with tools like vim, htop, btop, less
- **Long-running processes**: Start and monitor background tasks

## Getting Started

### List Available Sessions

```bash
tmux list-sessions
```

### Get Pane IDs

```bash
tmux list-panes -a -F "#{session_name}:#{pane_id}"
```

Pane IDs look like `%0`, `%1`, etc.

## Capturing Terminal Content

### Memory-Efficient Capture (RECOMMENDED)

Always limit captured lines to avoid filling context:

```bash
# Capture last 5 lines only
tmux capture-pane -p -t '%0' -S -5

# Capture last 10 lines
tmux capture-pane -p -t '%0' -S -10
```

The `-p` flag prints to stdout (no copy to clipboard). The `-S -N` flag captures last N lines from scrollback.

**Default: Use `-S -5` to `-S -10` for most operations.**

### Full Capture (AVOID unless necessary)

```bash
# Only use when full context needed
tmux capture-pane -p -t '%0' -S -200
```

### Clean Output

By default, output is clean text (no ANSI codes). Add `-e` flag if colors needed:

```bash
tmux capture-pane -p -e -t '%0' -S -5
```

## Executing Commands

```bash
# Send command to pane (like typing + Enter)
tmux send-keys -t '%0' 'your-command' Enter

# Wait briefly for execution
sleep 0.3

# Capture result (minimal)
tmux capture-pane -p -t '%0' -S -5
```

### Example: sudo command with password

```bash
# Send sudo command
tmux send-keys -t '%0' 'sudo apt update' Enter

# Wait for password prompt
sleep 0.5

# Check for password prompt
tmux capture-pane -p -t '%0' -S -3

# If password needed, send it
tmux send-keys -t '%0' 'YOUR_PASSWORD' Enter

# Wait and capture result
sleep 1
tmux capture-pane -p -t '%0' -S -10
```

### Example: Interactive TUI

```bash
# Start htop
tmux send-keys -t '%0' 'htop' Enter

# Navigate (send keys without Enter)
tmux send-keys -t '%0' 'q'  # Quit htop
```

## Session Management

```bash
# Create new session
tmux new-session -d -s session-name

# Create session in specific directory
tmux new-session -d -s session-name -c /path/to/dir

# Kill session
tmux kill-session -t session-name

# Attach to session (for human inspection)
tmux attach -t session-name
```

## Pane Management

```bash
# Split pane vertically (top/bottom)
tmux split-window -t '%0' -v

# Split pane horizontally (left/right)
tmux split-window -t '%0' -h

# Kill pane
tmux kill-pane -t '%0'
```

## Best Practices

1. **Use minimal capture**: Always use `-S -5` to `-S -10` to save context
2. **Wait after commands**: Use `sleep 0.3` to `sleep 1` after executing
3. **Dedicated sessions**: Create fresh sessions for AI work to avoid history baggage
4. **Confirm pane ID once**: Get pane ID at start of session, reuse for subsequent commands (pane IDs are stable unless pane is killed or tmux restarts)
5. **Check output**: Capture after execute to verify command success
6. **Clear history**: Use `clear` command or `tmux clear-history` to reset terminal

## Workflow Pattern

```
1. Get pane ID once → tmux list-panes -a -F "#{session_name}:#{pane_id}"
   (reuse same ID for all subsequent commands in this session)
2. Execute        → tmux send-keys -t '%X' 'command' Enter
3. Wait           → sleep 0.3
4. Capture result → tmux capture-pane -p -t '%X' -S -5
```

## Quick Reference

| Task | Command |
|------|---------|
| List sessions | `tmux list-sessions` |
| List panes | `tmux list-panes -a -F "#{session_name}:#{pane_id}"` |
| Capture output | `tmux capture-pane -p -t '%X' -S -5` |
| Send command | `tmux send-keys -t '%X' 'cmd' Enter` |
| Send keystroke | `tmux send-keys -t '%X' 'key'` |
| Create session | `tmux new-session -d -s name` |
| Kill session | `tmux kill-session -t name` |