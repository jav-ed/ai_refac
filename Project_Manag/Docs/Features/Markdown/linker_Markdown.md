# Markdown

This area documents the Markdown backend in `refac`: moving Markdown files, the images and other assets they point at, and folders of both, with every link that follows; and fixing the Markdown links to files that another backend (TypeScript, Python, Rust, Go, Dart, Kotlin) has moved. One engine does both. It reads Markdown as CommonMark, changes only link destinations, writes them the way the author wrote them, plans the whole change before touching the disk, and undoes everything if a step fails.

Code layout (`src/drivers/markdown/`): `parser/` finds destinations (CommonMark parser, `destination.rs`, raw HTML scanner `html.rs`); `href.rs` reads and writes one destination (percent-encoding, `./`, queries); `moves.rs` maps a path to where it ends up, folders included; `workspace.rs` finds the Markdown files (ignore-aware); `rewrite.rs` rewrites one file; `plan.rs` builds the plan; `apply.rs` carries it out with rollback; `documents.rs` and `mod.rs` are the entry points used by the CLI (`src/logic/markdown_links.rs` runs the pass after the other backends).

## Links

- [Supported Behavior](./supported_Behavior.md)
  What is moved, which link forms are updated, how a destination is written, the pass for links to code, and the safety rules.
- [Limits And Gaps](./limits_And_Gaps.md)
  What is not a link to refac (wiki-links, MDX imports, front matter), the boundaries of a move, style limits, and the test coverage.
- [Examples](./examples.md)
  Before and after scenarios: files, reference definitions, HTML, folders, percent-encoding, and links to code.
- [Markdown options](../../Investigation/markdown_Options.md)
  The investigation behind the engine: other tools (docmv, markmv, mdref, the VS Code Markdown language service, Marksman) tested on 13 link shapes, what each missed, and what is left to borrow (heading rename, dry run).
