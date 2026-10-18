# Web_Audit

A Bun CLI (`audit`) that checks pages for accessibility violations with axe-core. It has two independent modes that share only the report format.

| Mode | How | Needs | Use |
| --- | --- | --- | --- |
| `dist` | Reads the built HTML with axe-core and jsdom | no browser, no server | Right after a build, and in a pipeline |
| `dev` | Opens the pages in Playwright against a running server | a running server and a browser | Checks that need real rendering: contrast on computed styles, focus, scripts |

jsdom does not compute layout or all styles, so `dist` mode can miss what only a browser sees. Use `dev` mode when a result looks too clean.

## Run

Run from the project root so relative paths resolve.

```bash
audit dist ./dist
audit dist ./dist --tags 22aa --min-impact critical --output ./report.json
audit dev http://localhost:4321 --dist ./dist
audit report ./report.json --top 10 --min-impact critical
```

| Flag | Default | Meaning |
| --- | --- | --- |
| `--tags` | `21aa` | WCAG level: `2a`, `2aa`, `21aa` or `22aa`. |
| `--min-impact` | `serious` | The exit code is 1 when any violation is at this level or above: `minor`, `moderate`, `serious`, `critical`. |
| `--output <file>` | none | Save the JSON report. |
| `--dist <path>` (dev only) | `./dist` | Where `dev` mode discovers the URLs to visit. Without a `dist/` it falls back to a `pages.ts` config (`--config`). |

`audit report <file>` reads and filters a saved report without crawling again; `--top N` and `--min-impact` narrow it.

## Install

The tool is installed once as a global link from `Web_Audit_Home`: `bun link`. Then `audit` works from anywhere, as long as `~/.bun/bin` is on the `PATH` (`command -v audit` shows it). Without the link, run `bun src/index.ts dist ./dist` from `Web_Audit_Home`.

## In a pipeline

Not part of any pipeline template yet. As a check step it belongs after the build and the content checks and needs no browser: `audit dist <build folder> --min-impact serious`. A failed step stops the release like any other check.

## Deeper

For changing the tool: `Project_Manag/Docs/doc_Start.md` in `Web_Audit_Home` (architecture of the two modes, why jsdom and axe-core, URL discovery, the report schema, learnings about axe-core with Bun).
