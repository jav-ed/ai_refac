# Markdown Options

Which existing tools could move Markdown files and folders and fix the links, so that `refac` does not have to be that tool itself. This document records what was searched on 2026-10-07, what was run hands-on against one corpus of 13 link shapes, and why the backend stays an embedded engine on `pulldown-cmark`. Open it before replacing the Markdown engine, before adding heading rename, or before adding another document format.

**Status: decided and implemented.** The engine described in [Markdown](../Features/Markdown/linker_Markdown.md) was measured against the three tools that can be installed and run without an editor. All three missed or damaged links that the embedded engine handles; the details are below.

## What a Markdown move has to get right

A move changes two things: links that point at the moved path (from other files), and relative links inside the moved files (measured from a new place). Both have to cover every way a path is written in Markdown and in the HTML that READMEs mix in. The corpus used for the comparison has these 13 shapes:

1. inline link with a fragment, and an image
2. HTML `<img src>` and `<a href>`
3. a reference definition whose destination is on the next line
4. a folder link (`docs/`) and a `./` path
5. a destination in angle brackets with a space (`<docs/My Notes.md>`) and a percent-encoded one (`docs/My%20Notes.md`)
6. a link inside a code span and inside a fenced block (must not change)
7. a `?query` after the path
8. a link in a table cell
9. an autolink and a web URL that contain the same path text (must not change)
10. link text that continues on a second line
11. full, collapsed, and shortcut reference definitions
12. `srcset` with two candidates
13. a non-ASCII file name

Three moves were run on a copy of the corpus per tool: a file (`docs/guide.md` to `docs/manual/guide.md`), a folder (`docs` to `documentation`), and an image (`docs/img/logo.png` to `assets/logo.png`). The moved `guide.md` also links out (`../README.md`, `img/logo.png`, `My%20Notes.md`), so the inside links are tested too.

## Candidates

| Candidate | What it is | Runtime and license | Verdict |
|---|---|---|---|
| `docmv` 0.1.0 | CLI that moves Markdown files and rewrites links, with `plan`, `apply`, `audit`, `explain` | Python 3.10+, no dependencies, MIT, first release 2026-06-17 | Run. Misses several shapes, see below. Its own README states the tokenizer is line oriented and skips multi-line links and definitions on purpose |
| `markmv` 1.41.7 | CLI, REST API, MCP server, and library with move, split, join, merge, index, heading refactor, link graph | Node.js 18+. `package.json` and `LICENSE` say MIT while its README says CC BY-NC-SA 4.0 (non-commercial); the contradiction alone is a reason not to embed it | Run. Not scanning the whole project for a single-file move, misses reference definitions and HTML |
| `mdref` 0.4.4 | Rust crate and CLI: `find`, `mv`, `rename`, built on `comrak` | MIT | Run. It crashed on an angle-bracket destination and, worse, emptied an image destination |
| `vscode-markdown-languageservice` | The library behind VS Code's "update links on file move" and heading rename. Returns a `WorkspaceEdit` for renamed files | Node.js library, MIT, CommonMark only, needs an `IMdParser` implementation | Not run. Needs a Node runtime and a parser adapter; would replace a Rust function with a second runtime. Its feature list (header rename, reference links, fragments) is a reference for what to cover |
| Marksman | Markdown LSP: completion, go to definition, find references, rename, diagnostics. Inline, reference, and wiki links. Self-contained binary, MIT | Rename is heading-and-wiki-link oriented; its docs do not say it handles `workspace/willRenameFiles` | Not run. Candidate to revisit for heading rename and wiki-links, not for file moves |
| markdown-oxide | Markdown LSP for personal knowledge bases, Rust, Apache-2.0 | Focused on wiki-links and daily notes; file-move link updates are not documented | Not run |
| Editor features (Obsidian, VS Code, Zed) | Update links when a file is moved inside the editor | Need the editor | Rejected: an agent has no editor |

## Hands-on results

Folder move (`docs` to `documentation`), every tool run on the same corpus. A tick means the link now leads to the moved file; a cross means the link was left pointing at the old path or was broken.

| Shape | refac | docmv | markmv | mdref |
|---|---|---|---|---|
| inline link and image | ok | ok (adds `./`) | ok (adds `./`) | crashed |
| HTML `<img src>`, `<a href>` | ok | missed | missed | crashed |
| reference definition, next-line destination | ok | missed | missed | crashed |
| full, collapsed, shortcut definitions | ok | ok (adds `./`) | missed | crashed |
| folder link `docs/` | ok | missed | missed | crashed |
| `<docs/My Notes.md>` | ok | ok (adds `./`) | missed | crashed |
| `docs/My%20Notes.md` | ok | missed | missed | crashed |
| `?query` | ok | ok (adds `./`) | missed | crashed |
| table cell | ok | ok (adds `./`) | ok (adds `./`) | crashed |
| link text over two lines | ok | missed | missed | crashed |
| `srcset` | ok | missed | missed | crashed |
| non-ASCII name | ok | ok (adds `./`) | ok (adds `./`) | crashed |
| code span, fence, autolink, web URL | untouched | untouched | untouched | crashed |
| links inside the moved folder | untouched, correct | broken: `[notes](../docs/My%20Notes.md)` | broken: same | crashed |

Findings that are not in the table:

- **Single-file move, `markmv`:** `Found 0 files that reference docs/guide.md`. A `README.md` one folder above the moved file was never scanned, so every inbound link stayed stale. Folder moves did find it.
- **Image move, `mdref`:** `![logo](docs/img/logo.png)` became `![logo]()`. The tool silently emptied the destination, which is the worst failure class: no error and a lost path.
- **Folder move, `mdref`:** `Error: could not find link '](docs/My Notes.md)' in line 7`. It stops at the first destination it cannot locate again.
- **Style:** `docmv` and `markmv` write `./` in front of every path they touch (`docmv` has `--style bare` to turn that off); `mdref` removes `./` that the author wrote. `refac` keeps the author's habit.
- **Folders:** none of the three updated the inside links of a moved folder correctly. After the move `docs/` is gone, yet `docmv` and `markmv` wrote `../docs/My%20Notes.md` into the moved file.
- **Percent encoding:** none decoded `%20` when resolving, so a link to `My Notes.md` written as `My%20Notes.md` was missed or damaged.
- **Writes:** no tool documents a rollback for a move that fails halfway (`mdref` calls its update atomic; not verified); `refac` journals the moves and writes and restores them, and a test pins that.

## Decision

Keep the embedded engine. Reasons, in order of weight:

1. **Correctness on the shapes that exist in real READMEs.** HTML references, multi-line definitions, folder links, percent-encoding, and moved folders are normal in project documentation, and each of them was wrong in at least two of the three tools.
2. **No second runtime.** `refac` is one binary. `docmv` needs Python, `markmv` and the VS Code service need Node.
3. **One engine for both jobs.** The same link rewriting runs when Markdown files move and after a TypeScript, Python, Go, Rust, Dart, or Kotlin file moves (the README that points at `src/app.py`). The existing tools only know Markdown-to-Markdown.
4. **Safety properties** the tools lack: a CommonMark parser decides what is a link; refac replaces only the destination bytes and checks them against the parser's own destination; style is preserved; a failed move is rolled back; and a link is only changed when its old target is gone and the new one is there.

`pulldown-cmark` was kept over `comrak` (used by `mdref`) because it reports source ranges for events and for reference definitions (`reference_definitions()` with spans), which is what makes destination-only rewriting possible.

## What to take from the others

- **Dry run and audit** (`docmv plan`, `audit`; `mdref --dry-run`; `markmv --dry-run`). The engine already builds the full plan before touching the disk, so `refac move --dry-run` for documents is cheap; it is not exposed yet because the move command has no dry-run flag across languages.
- **Heading rename with `#slug` links.** `markmv refactor-headings`, Marksman, and the VS Code service all do it. A Rust implementation would use `pulldown-cmark` for headings and the `github-slugger` crate for GitHub's anchor algorithm (including `-1` suffixes for duplicates). It is the natural next feature for `refac rename` on `.md` files.
- **Wiki-links and Obsidian vaults** (`markmv --obsidian`, Marksman, markdown-oxide) are a separate link model resolved by note name; not supported, documented as a limit.
- **`explain`** (`docmv explain`) shows how each link in a file resolves; a similar report would help an agent debug a link that was not changed.

## Reproduce

The corpus is created by a 46-line script (the 13 shapes above plus a `git init`) and each tool is run as `docmv apply SRC DST`, `markmv move SRC DST`, `mdref mv SRC DST`, and `refac move --project-path . --source-path SRC --target-path DST`. The `refac` result is pinned by `tests/markdown_corpus.rs`, which holds the same corpus and the exact expected text for the three moves.

## Sources

- [docmv on PyPI](https://pypi.org/project/docmv/)
- [markmv README](https://cdn.jsdelivr.net/npm/markmv@1.40.0/README.md)
- [mdref on docs.rs](https://docs.rs/mdref/0.4.1/mdref)
- [VS Code Markdown Language Service](https://github.com/microsoft/vscode-markdown-languageservice)
- [Introducing the Markdown Language Server (VS Code blog)](https://code.visualstudio.com/blogs/2022/08/16/markdown-language-server)
- [Marksman](https://github.com/artempyanykh/marksman) and its [feature list](https://github.com/artempyanykh/marksman/blob/master/docs/features.md)
- [markdown-oxide](https://github.com/Feel-ix-343/markdown-oxide)
- [github-slugger for Rust](https://docs.rs/github-slugger)
