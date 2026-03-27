# Doc Start

This repo is a CLI-first refactoring tool. Its job is to move files and update affected references so projects stay consistent after structural changes. The current runtime surface is the `refac` CLI, and the implementation uses language-specific backends for TypeScript/JavaScript, Python, Markdown, Rust, Go, and Dart.

Use this file as the top-level handoff. Do not paste large doc contents into agent context by default. Read only the linked files that are relevant to the task at hand.

Operational decision: after building `refac`, we want it available via `~/Progs/Bins`. During active development, the preferred setup is a symlink from `~/Progs/Bins/refac` to the built binary. That keeps the command stable while letting rebuilt binaries take effect without reinstall steps beyond rebuilding.

## Quick Reference — What to Read for What

| Question / Task | Read |
|---|---|
| How do I run `refac`? What arguments does it take? | `refac --help` or `refac move --help` |
| What languages / file types are supported? | [Capabilities & Supported Languages](./Descr/abilties.md) |
| How does Markdown move support work? What are its limits? | [Markdown Feature Docs](./Features/Markdown/linker_Markdown.md) |
| How does TypeScript / JS move work? What are its limits? | [TypeScript Feature Docs](./Features/TypeScript/linker_TypeScript.md) |
| How does Python move work? What are its limits? | [Python Feature Docs](./Features/Python/linker_Python.md) |
| How does Go move work? Batch session architecture? Whole-package behaviour? | [Go Feature Docs](./Features/Go/linker_Go.md) |
| How does Rust move work? Same-dir vs cross-dir? Shim strategy? | [Rust Feature Docs](./Features/Rust/linker_Rust.md) |
| What are the limits per language? (summary table) | [Capabilities & Limits](./Descr/abilties.md) §3 |
| Can I move a folder/directory? | TypeScript only — directory sources are routed to the TypeScript driver. All other languages require individual files. |
| Why is refac slow or hanging on a large TS project? | Large projects (>500 files) skip full reference resolution; see [Capabilities](./Descr/abilties.md) |
| How do I build and install the binary? | [Developer & Build Guide](./Guides/dev_guide.md) |
| How do I run the tests / debug a failure? What tests exist for batch moves? | [Testing & Debugging Guide](./Guides/Testing_and_Debugging.md) |
| Why is `ty` not used for Python? | [ty Python Refactoring Notes](./Research/ty_python_refactoring.md) |
| Why was backend X chosen over Y? | [Tool Research Report](./Research/tool_Research_Report.md) |
| What is the overall goal/scope? | [Project Goal](./Descr/goal.md) |
| Where does the public docs site live? What is the content structure? | [Docs Site](./Guides/linker_DocSite.md) |
| Where is the Starlight source / local reference docs? | `Repos/starlight/` — cloned at `--depth=1` from github.com/withastro/starlight |
| How do I add a new page or language to the docs site? | [Docs Site — Adding pages](./Guides/linker_DocSite.md#adding-a-new-page) |
| How do I build and deploy the docs site to refac.javedab.com? | [Deploy Guide](./Guides/deploy_docs.md) |
| What is the difference between internal docs and the public docs site? | [Docs Site — Relationship to internal docs](./Guides/linker_DocSite.md#relationship-to-internal-docs) |
