---
name: web-comp-tools
description: "Home of the shared web tools that static-site pipelines call: Compressor (.br, .gz and .zst files), Og_Image (social cards), Index_Now (IndexNow notifications) and Web_Audit (accessibility audit), with where each one lives, how to run it and which pipeline step uses it. Use when a task needs to compress a build, generate OG images, notify search engines, audit a built site for accessibility, or find a tool's folder, command or flags."
---

# web-comp-tools

The tools below are separate repositories that every web project reuses instead of carrying its own copy. This skill is the one place that says what each tool does, how to run it, and what goes wrong. The skills for sites and pipelines (`astro`, `fuma`, `ci-cd`) do not repeat tool text: they say "read the `web-comp-tools` skill" and keep only what belongs to their framework or platform.

Names such as `Compressor_Home` stand for a folder or binary. Their real values are in [path_File.md](path_File.md), the only file in this skill that contains a path. Paste its block into the shell once and every command below works as written.

## Why it is built this way

- **One home for tool facts.** A fact written in several skills ends up wrong in one of them, and the copies of one pipeline script in different skills already differ from each other. Each tool is described once, here, and the other skills link to it.
- **One path file.** Folders move, and a path written in many files breaks in many places. Every path lives in `path_File.md`, the pages use names, and one edit fixes the skill.
- **Operator pages here, developer docs in the tool.** Whoever runs a tool needs the command, the inputs, the result and the failures. Whoever changes it needs the architecture. This skill holds the first and each page ends with a "Deeper" pointer to the second, which stays next to the code and changes in the same commit.
- **Tools only.** Rules about media are not tools and are not here: the formats, loading and alt text of images are the `web-media` skill, and the social cards this skill renders are the one media topic that stays, because a tool makes them.
- **Other skills keep what is theirs.** The Astro manifest builder, the Fumadocs route head and the Woodpecker step templates depend on that framework or platform, so they stay in their skills. The tool itself is here.

## The tools

| Tool | What it does | Home name | Build or install, then run |
| --- | --- | --- | --- |
| Compressor | Writes `.br`, `.gz` and `.zst` files next to a build's text files | `Compressor_Home` | `cargo build --release` there, run `Compressor_Bin` |
| Og_Image | Renders 1200 by 630 social cards from a manifest the site wrote | `Og_Image_Home` | `bun install` there, run `bun src/main.js` inside it |
| Index_Now | Tells IndexNow which pages really changed between two releases | `Index_Now_Home` | `just build` there, run `Index_Now_Bin` (`indexnow`); `just deploy` installs it on g12 |
| Web_Audit | Accessibility audit of a built `dist/` or a dev server (axe-core) | `Web_Audit_Home` | `bun link` there once, then run `audit` |
| CI_CD | The Woodpecker and Forgejo runtime that runs the pipelines | `CI_CD_Home` | owned by the `ci-cd` skill, not described here |

Check what is installed with `command -v audit indexnow compressor`, or run the check at the end of [path_File.md](path_File.md). A missing tool is built from its home with the command in the last column; never guess a path.

## Which tool for which step

| Step | Tool | Page |
| --- | --- | --- |
| Check the built site for accessibility violations | Web_Audit, `dist` mode (no browser) | [Web_Audit](Web_Audit/web_Audit.md) |
| Per-page social cards | Og_Image, after the build and before compression | [Og_Image](Og_Image/og_Image.md), and [what every site does](Og_Image/Per_Page_Cards/site_Checklist.md) |
| One social card for a small site | no tool: an HTML card rendered once | [One card](Og_Image/Single_Card/single_Card.md) |
| Precompress the build | Compressor, after every file that goes into the build exists | [Compressor](Compressor/compressor.md) |
| Tell search engines about changed pages after a deploy | Index_Now, needs a sitemap | [Index_Now](Index_Now/index_Now.md) |
| Install, build, check, deploy, roll back, trigger | the `ci-cd` skill | |

## Common case

Every tool answers `--help` with the full flag list, so this skill lists only what a normal run needs.

```bash
"$Compressor_Bin" --dir dist --no-color                            # sidecars next to every text file; needs an uncompressed dist
audit dist ./dist                                                  # exit 1 on a serious violation; --output report.json saves it
"$Index_Now_Bin" compare-releases --site-url https://example.com/ --sitemap sitemap-index.xml \
    --previous-root <old release> --current-root <new release>     # offline: shows what would be submitted
(cd "$Og_Image_Home" && bun src/main.js --project_Root /abs/path/to/site)   # needs the site's manifest first
```

A pipeline never runs these by hand: its steps call them with the values of its `env.env`.

## Navigation

- [path_File.md](path_File.md): every path of the skill in one block, the check that proves they exist, and what to do when a folder moves or a tool is added. Open it first when a command needs a folder or binary.
- [Compressor](Compressor/compressor.md): precompressing a build for Caddy, flags that matter, the cache and why it needs the same filesystem, the "Destination already exists" failure, and how the compress step calls it.
- [Og_Image](Og_Image/og_Image.md): the per-page card tool: which card (family and variant) fits a site, how to run it (arguments, output folder, size limits), the pipeline step, and the pointers to the two folders below. Open it to run or choose a card.
- [Per-page cards, what every site does](Og_Image/Per_Page_Cards/site_Checklist.md): the rules that hold on any stack (generated and never committed, one name rule, no fallback card, display copy, WebP, head tags, verify), the seven parts every site needs, and the three layers of knowledge (this skill, the stack skill, the site's own `og_Cards.md`). Open it when adding cards to a site or reviewing a site's cards. [Site page template](Og_Image/Per_Page_Cards/site_Page_Template.md): the skeleton of the page each site keeps about its own cards.
- [One card for the whole site](Og_Image/Single_Card/single_Card.md): the no-tool method for a small site: an HTML card rendered once and committed, with its two copyable files in the same folder.
- [Index_Now](Index_Now/index_Now.md): when IndexNow is worth adding, prerequisites (sitemap, retained releases, ownership key), the five subcommands, and what the binary does when something is wrong. The deploy adapter (variables, bootstrap state, recovery) is in the `ci-cd` skill.
- [Web_Audit](Web_Audit/web_Audit.md): running the axe-core audit in `dist` or `dev` mode, WCAG levels, the exit-code threshold, and reading a saved report.
