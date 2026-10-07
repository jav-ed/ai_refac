# External Reference Repos

Shallow clones of upstream projects kept under the gitignored `Repos/` folder at the repository root, so their source and docs can be read without network calls. Nothing inside `Repos/` is committed; this file is the source of truth for what should be present. To restore a missing clone, run its `Clone:` command from the repository root. Entries without a recorded reason were already present in `Repos/` when this manifest was created on 2026-10-07; their lines say what the project is, so a reader can judge whether it matters for the task.

## Language servers and refactoring engines

- **rust-analyzer**: Rust language server, the engine behind Rust file renames and the embedded HIR used by `move-module`.
  - URL: https://github.com/rust-lang/rust-analyzer
  - Clone: `git clone --depth 1 https://github.com/rust-lang/rust-analyzer Repos/rust-analyzer`
- **tools**: Go tools repository containing gopls, the engine behind Go moves.
  - URL: https://github.com/golang/tools
  - Clone: `git clone --depth 1 https://github.com/golang/tools Repos/tools`
- **kotlin-lsp**: JetBrains' official Kotlin language server (read-only mirror, partly closed-source). Candidate engine for Kotlin file moves and symbol rename; its source shows the `workspace/willRenameFiles` handler that runs IntelliJ's move refactoring. See the Kotlin options investigation.
  - URL: https://github.com/Kotlin/kotlin-lsp
  - Clone: `git clone --depth 1 https://github.com/Kotlin/kotlin-lsp Repos/kotlin-lsp`
- **kotlin-language-server**: community Kotlin language server, deprecated in favour of the official one. Cloned only to document why it was rejected.
  - URL: https://github.com/fwcd/kotlin-language-server
  - Clone: `git clone --depth 1 https://github.com/fwcd/kotlin-language-server Repos/kotlin-language-server`
- **rewrite-kotlin**: OpenRewrite's Kotlin recipes and visitors. Cloned only to document why it was rejected as an interactive rename and move engine.
  - URL: https://github.com/openrewrite/rewrite-kotlin
  - Clone: `git clone --depth 1 https://github.com/openrewrite/rewrite-kotlin Repos/rewrite-kotlin`
- **pyrefly**: Python type checker and language server used as the fallback Python backend.
  - URL: https://github.com/facebook/pyrefly
  - Clone: `git clone --depth 1 https://github.com/facebook/pyrefly Repos/pyrefly`
- **ty**: Astral's Python type checker, studied and rejected for Python moves (see the Python research note).
  - URL: https://github.com/astral-sh/ty
  - Clone: `git clone --depth 1 https://github.com/astral-sh/ty Repos/ty`
- **markmv**: Markdown file mover with link updates, a comparison point for the Markdown backend.
  - URL: https://github.com/ExaDev/markmv
  - Clone: `git clone --depth 1 https://github.com/ExaDev/markmv Repos/markmv`
- **marksman**: Markdown language server with link and rename support, a comparison point for the Markdown backend.
  - URL: https://github.com/artempyanykh/marksman
  - Clone: `git clone --depth 1 https://github.com/artempyanykh/marksman Repos/marksman`
- **markdown-oxide**: Markdown language server with link and rename support, a comparison point for the Markdown backend.
  - URL: https://github.com/Feel-ix-343/markdown-oxide
  - Clone: `git clone --depth 1 https://github.com/Feel-ix-343/markdown-oxide Repos/markdown-oxide`
- **iwe**: Markdown knowledge-base language server, a comparison point for the Markdown backend.
  - URL: https://github.com/iwe-org/iwe
  - Clone: `git clone --depth 1 https://github.com/iwe-org/iwe Repos/iwe`

## Real-world TypeScript projects

- **tanstack-router**: large TypeScript monorepo (about 6,200 TS/TSX files across many packages) used as a realistic scale target. The symbol-rename engine comparison ran on a copy of its packages.
  - URL: https://github.com/TanStack/router
  - Clone: `git clone --depth 1 https://github.com/TanStack/router Repos/tanstack-router`

## Public docs site references

- **fumadocs**: documentation framework used by the public `Refac_Docs` site.
  - URL: https://github.com/fuma-nama/fumadocs
  - Clone: `git clone --depth 1 https://github.com/fuma-nama/fumadocs Repos/fumadocs`
- **starlight**: Astro documentation framework, a reference for docs-site structure.
  - URL: https://github.com/withastro/starlight
  - Clone: `git clone --depth 1 https://github.com/withastro/starlight Repos/starlight`
- **pagefind**: static search index used by documentation sites.
  - URL: https://github.com/CloudCannon/pagefind
  - Clone: `git clone --depth 1 https://github.com/CloudCannon/pagefind Repos/pagefind`
- **geist-font**: Geist typeface source, a typography reference for the docs site.
  - URL: https://github.com/vercel/geist-font
  - Clone: `git clone --depth 1 https://github.com/vercel/geist-font Repos/geist-font`
- **maple-font**: Maple Mono typeface source, a typography reference for the docs site.
  - URL: https://github.com/subframe7536/maple-font
  - Clone: `git clone --depth 1 https://github.com/subframe7536/maple-font Repos/maple-font`

## Protocol references

- **rust-sdk**: official Rust SDK for the Model Context Protocol, a reference for the earlier MCP server surface.
  - URL: https://github.com/modelcontextprotocol/rust-sdk
  - Clone: `git clone --depth 1 https://github.com/modelcontextprotocol/rust-sdk Repos/rust-sdk`
