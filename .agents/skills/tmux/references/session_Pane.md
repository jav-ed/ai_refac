# Session & Pane Management

## Sessions

```bash
# Create detached session
tmux new-session -d -s session-name

# Create in specific directory
tmux new-session -d -s session-name -c /path/to/dir

# List sessions
tmux list-sessions

# Attach (for human interaction / password entry)
tmux attach -t session-name

# Kill session
tmux kill-session -t session-name
```

## Panes

```bash
# List all panes with IDs
tmux list-panes -a -F "#{session_name}:#{pane_id}"

# Split vertically (top/bottom)
tmux split-window -t '%X' -v

# Split horizontally (left/right)
tmux split-window -t '%X' -h

# Kill pane
tmux kill-pane -t '%X'
```

## TUI Applications

Send keystrokes without Enter for navigation:

```bash
# Start htop
tmux send-keys -t '%X' 'htop' Enter

# Quit (no Enter needed)
tmux send-keys -t '%X' 'q'
```

## Best Practices

- Create a **dedicated named session** for AI work (e.g., `diag`) to avoid history noise from other sessions
- Get the pane ID once at the start and reuse it — IDs are stable unless the pane is killed or tmux restarts
- Use `tmux clear-history -t '%X'` to reset scrollback before a batch of commands
