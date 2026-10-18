# Compressor

A Linux-only Rust CLI that writes `.br` (Brotli), `.gz` (gzip) and `.zst` (Zstd) files next to the text files of a finished build, so Caddy can serve them without compressing on every request. It works on any static build output, Astro and Fumadocs alike.

It handles `html`, `css`, `js`, `mjs`, `cjs`, `json`, `svg`, `xml`, `txt` and `wasm` files. Images and fonts are left alone. The list is fixed in the code.

## Run

```bash
"$Compressor_Bin" --dir dist --no-color
```

All three codecs are on, at Brotli 11, gzip 12 and Zstd 19, with the thread count chosen automatically. `--no-color` keeps CI logs clean. Run it last, after every file that belongs in the build exists (OG images included), and only on an uncompressed folder.

## Flags that matter

| Flag | Use |
| --- | --- |
| `--dir <path>` | Required. The build output folder, for example `dist` or `dist/client`. |
| `--no-brotli`, `--no-gzip`, `--no-zstd` | Turn one codec off. |
| `--brotli-level`, `--gzip-level`, `--zstd-level` | Change a level (0 to 11, 1 to 12, 1 to 22). |
| `--threads <N or auto>` | Default `auto`, which leaves one core free. |
| `--no-cache`, `--force-recompress` | Skip the cache, or ignore its hits and recompress everything. |
| `--cache-dir <path>`, `--allow-copy-fallback` | Put the cache elsewhere, or copy when a hardlink is impossible. |

`"$Compressor_Bin" --help` lists everything.

## The cache

The cache is on by default and lives in `<parent of --dir>/Cache/Super_Compressor/`. Files whose content did not change come back as hardlinks instead of being compressed again, so a rebuild of a mostly unchanged site is fast. Hardlinks need the cache and the build folder on the same filesystem.

## In a pipeline

The compress step reads the binary path from `CI_COMPRESSOR_BIN` in the project's `env.env`, refuses to continue when it is missing or not executable, and runs `"$CI_COMPRESSOR_BIN" --dir <build folder> --cache`. The step template belongs to the `ci-cd` skill; the path value comes from `Compressor_Bin` in [path_File.md](../path_File.md).

## Failures

| Message or exit | Cause | Fix |
| --- | --- | --- |
| `Destination already exists (invariant violation)`, exit 1 | A `.br`, `.gz` or `.zst` file is already there: the folder was compressed before and the build did not clean it | Rebuild, then compress once. Never compress twice. |
| Missing or invalid `--dir` | The build did not run or the path is wrong | Run the build step first; check the path. |
| Hardlink restore or store fails | Cache and build folder are on different filesystems | Put `--cache-dir` on the same filesystem, or add `--allow-copy-fallback` on purpose. |
| A corrupt cache index | Interrupted run | Nothing: it is treated as empty and rebuilt, with a warning. |

## Deeper

For changing the tool, not for using it: `Project_Manag/Docs/cli_Reference.md` (every flag and exit rule), `integration_Guide.md`, `troubleshooting.md` and `Architecture/workflow.md` in `Compressor_Home`.
