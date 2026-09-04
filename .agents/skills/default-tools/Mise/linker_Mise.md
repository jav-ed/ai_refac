# Mise

Use mise only to install and update selected system tools. Just owns repository tasks. Normal commands must not depend on mise activation, shims, or project configuration.

## Rules

- Declare tools as `latest` in `Project_Manag/Tools/Mise/system_Tools.toml` and comment their purpose.
- Use `just system-tools-install` for setup and `just system-tools-update` for upgrades.
- Treat `/etc/mise/config.toml` as an installed copy, not the file to edit.
- Install under `/usr/local/share/mise/installs` with `mise install --system`.
- Link each approved executable into `/usr/local/bin`.
- Run system maintenance from `/` with a clean environment.
- Normalize permissions because g12 uses umask `077`.
- Verify commands with an empty environment and the intended service account.

Do not add mise shims to `PATH`, use `mise activate`, or use mise as a task runner.

## Reference

[Platform tool management](../../../../Project_Manag/Docs/Architecture/Platform_Essentials/tool_Management.md): canonical commands, system layout, update caveat, cleanup, and verification.
