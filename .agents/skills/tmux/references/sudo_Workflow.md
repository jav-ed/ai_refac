# sudo Workflow

Claude Code runs in a headless subprocess with no TTY — `sudo` cannot prompt for a password there. The tmux session provides a real TTY, but the user must be attached to type the password.

## Flow

1. Tell the user to attach if not already: `tmux attach -t <session>`
2. Send the sudo command — the password prompt appears in their terminal
3. Use file+sentinel with a longer poll timeout to wait for password entry + completion

```bash
rm -f /tmp/tmux_out.txt /tmp/tmux_done.txt

# Send command — user will see the password prompt in their attached terminal
tmux send-keys -t '%X' 'sudo systemctl restart someservice > /tmp/tmux_out.txt 2>&1; echo ok > /tmp/tmux_done.txt' Enter

# Poll longer (20s) to give user time to type password
for i in $(seq 1 40); do [ -f /tmp/tmux_done.txt ] && break; sleep 0.5; done

cat /tmp/tmux_out.txt
```

## Credential Caching

Once the user types their password, `sudo` caches it for ~15 minutes (default). Subsequent sudo commands in the same tmux pane will not prompt again — the file+sentinel pattern works without interruption.

## Alternative: NOPASSWD Sudoers Rule

For ongoing admin work, add a NOPASSWD rule so sudo never prompts. This removes all friction — file+sentinel works without the user needing to be attached at all.

```bash
echo 'username ALL=(ALL) NOPASSWD: ALL' | sudo tee /etc/sudoers.d/username-nopasswd
```

Remove when no longer needed:

```bash
sudo rm /etc/sudoers.d/username-nopasswd
```
