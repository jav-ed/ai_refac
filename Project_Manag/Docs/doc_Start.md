# Doc Start

This repo is a CLI-first code refactoring tool. Its job is to move files and update affected references so projects stay consistent after structural changes. The current runtime surface is the `refac` CLI, and the implementation uses language-specific backends for TypeScript/JavaScript, Python, Rust, Go, and Dart.

Use this file as the top-level handoff. Do not paste large doc contents into agent context by default. Read only the linked files that are relevant to the task at hand.

Operational decision: after building `refac`, we want it available via `~/Progs/bin`. During active development, the preferred setup is a symlink from `~/Progs/bin/refac` to the built binary. That keeps the command stable while letting rebuilt binaries take effect without reinstall steps beyond rebuilding.

## Additional Docs

- [Developer & Build Guide](./dev_guide.md)
  Build flow, local development workflow, install-to-PATH decision, and the generated testbed.
- [Testing & Debugging Guide](./Testing_and_Debugging.md)
  Testing notes and debugging guidance for the CLI workflow.
- [Capabilities & Supported Languages](./Descr/abilties.md)
  Overview of supported language backends and current tool behavior.
- [Project Goal](./Descr/goal.md)
  High-level project objective and scope.
- [Tool Research Report](./Research/tool_Research_Report.md)
  Background research on backend choices and tradeoffs across languages.
- [ty Python Refactoring Notes](./Research/ty_python_refactoring.md)
  Reference notes about why `ty` is not currently used as the Python move/rename backend.
