# Output Capture

## ✅ BEST: File + Sentinel

Redirect output to a file, use a sentinel to signal completion. Poll instead of sleeping blindly.

```bash
rm -f /tmp/tmux_out.txt /tmp/tmux_done.txt
tmux send-keys -t '%X' 'your-command > /tmp/tmux_out.txt 2>&1; echo ok > /tmp/tmux_done.txt' Enter

for i in $(seq 1 20); do [ -f /tmp/tmux_done.txt ] && break; sleep 0.5; done
cat /tmp/tmux_out.txt
```

**Why:** Reliable completion detection, captures full output regardless of length, no ANSI garbage.

## ✅ OK: capture-pane (Short Output Only)

```bash
tmux send-keys -t '%X' 'your-command' Enter
sleep 0.3
tmux capture-pane -p -t '%X' -S -10
```

Always limit with `-S -N`. Use `-S -5` to `-S -10` by default to avoid filling context.

## ❌ AVOID: pipe-pane

Pipes raw terminal output including ANSI escape codes — output is unreadable. Do not use for capturing command output.
