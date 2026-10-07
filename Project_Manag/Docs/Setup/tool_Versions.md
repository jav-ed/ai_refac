# Tool versions

Every tool refac builds on or drives, the version this checkout was verified with, and where that version is pinned. It was last audited on 2026-10-07 against the registries (crates.io, npm, PyPI, Go, Dart, Gradle, Maven, the Kotlin server release list). Each row marked "latest" was at its newest stable release on that day; a bump is only recorded after the whole suite passed on it.

The point of this file is to make the next audit quick: the pin column says where to change a version, the notes column says what broke or constrains a bump.

## Build and Rust crates

| Tool | Version | Pinned in | Notes |
|---|---|---|---|
| Rust toolchain | 1.99.0 (latest stable) | `rust-toolchain.toml`, `rust-version` in `Cargo.toml` | The toolchain also supplies rustfmt and the rust-analyzer binary used for same-directory `.rs` renames. |
| `ra_ap_hir`, `ra_ap_ide`, `ra_ap_load-cargo`, `ra_ap_project_model`, `ra_ap_syntax`, `ra_ap_vfs` | 0.0.357 (latest) | `Cargo.toml`, exact `=` pins | These crates have no stable API; every release can break the Rust `move-module` code, so they move together and only after `rust_module_move` and `rust_move` pass. |
| `clap_mangen` | 0.3 (latest) | `Cargo.toml` | Used only for the `man` page output in `src/cli.rs`. |
| `lsp-types` | 0.97.0 (latest) | `Cargo.toml` | The type crate for the LSP wire structures. |
| All other crates | newest compatible | `Cargo.lock` | `cargo update` takes semver-compatible bumps. |

`Cargo.lock` is committed on purpose. `ra-ap-rustc_lexer` refuses to compile when `unicode-ident` and `unicode-properties` carry different Unicode versions (a compile-time panic), which happened once with a blanket `cargo update`. After any `cargo update` build before committing; if that panic appears, lower `unicode-ident` with `cargo update -p unicode-ident --precise <older>` until the two agree.

## TypeScript helper (`scripts/`)

| Tool | Version | Pinned in | Notes |
|---|---|---|---|
| Bun | 1.4.2 | not pinned, `@types/bun` follows it | Runs the helper and its tests. |
| `oxc-parser` | 0.153.0 (latest) | `scripts/package.json` | Import and symbol parsing. |
| `oxc-resolver` | 11.24.2 (latest) | `scripts/package.json` | Module resolution. |
| `typescript` | 6.0.3 | `scripts/package.json` | The move planner's config reader and resolver (`scripts/TypeScript/project.ts`, `resolver.ts`) import its JavaScript API. Version 7 ships a native binary and only an `unstable/*` JavaScript API, so 6.0.3 stays next to `typescript-native`. |
| `typescript-native` | 7.0.2 (latest) | `scripts/package.json` | The native TypeScript 7 language server behind `refac rename`. |

## Language servers and the tools they need

| Tool | Version | Where it comes from | Notes |
|---|---|---|---|
| Kotlin language server | `ILS-263.6379.0` (newest release) | `REFAC_KOTLIN_SERVER`, see [Kotlin server setup](kotlin_Server.md) | Early access build; the license key it carries runs out on 2026-10-30. |
| gopls | v0.23.0 (latest) | `go install golang.org/x/tools/gopls@latest` | Tested with Go 1.27.1 (latest) and 1.24.7. |
| Dart SDK | 3.13.5 (latest) | the Dart SDK's `dart language-server` | Needs `.dart_tool/package_config.json` for `package:` imports. |
| Rope | 1.15.0 (latest) | `pip install rope` | First choice for Python moves. |
| pyrefly | 1.3.2 (latest) | `pip install pyrefly` | Fallback behind Rope; the ignored test `test_pyrefly_move_rewrites_importer` exercises it. |

## Kotlin and Android fixtures

| Tool | Version | Pinned in | Notes |
|---|---|---|---|
| Gradle (both fixtures) | 9.8.1 (latest) | `gradle/wrapper/gradle-wrapper.properties` of `tests/fixtures/kotlin/*` | Regenerate with `./gradlew wrapper --gradle-version <v>` in a copy and copy the wrapper files back. |
| Kotlin Gradle plugin | 2.4.20 (latest stable) | `tests/fixtures/kotlin/jvm_project/build.gradle.kts` | |
| Android Gradle plugin | 9.4.1 (latest stable) | `tests/fixtures/kotlin/android_project/build.gradle.kts` | Newer 9.5 builds are alpha only. |
| Android SDK | platform 36, build-tools 36.0.0 | `ANDROID_HOME` | Not in git; the Android tests panic without it. |
| JDK | 21 here, 17 or newer required | the machine | The Kotlin server brings its own runtime; Gradle uses the machine JDK. |

## Alternatives that were weighed and not taken

Written so the next audit does not repeat the comparison; each line says what would change the answer.

- **`lsp-types` 0.97.0 against its fork `ls-types` 0.0.6** (`tower-lsp-community`, last release 2026-03). `lsp-types` has not had a release since 2024-06, but it is still the crate most of the ecosystem uses and the LSP 3.17 wire types it models have not changed. `ls-types` models the same 3.17 types and offers 3.18 only behind an unstable `proposed` feature, and it is a 0.0.x crate. refac needs nothing from 3.18, so the swap would be a rename of every import for no behaviour gain. Revisit when a language server refac drives needs a 3.18 type, or when `lsp-types` breaks on a new compiler.
- **Rope for Python moves** (1.15.0, latest). It stays the first choice because it updates imports more reliably for file moves than Pyrefly's `willRenameFiles` (the comment in `src/drivers/python/mod.rs`); Pyrefly is only the fallback.
- **Markdown**: the link scanner was replaced by `pulldown-cmark` 0.13.4 (see [Limits and gaps](../Features/Markdown/limits_And_Gaps.md)). It reports source byte offsets for every event and exposes reference definitions with their spans, which is what the destination reader needs, and an older version of it is already in the dependency tree through `ra_ap_ide`. Other CommonMark crates were not trialled; `comrak` (used by the `mdref` crate) offers no destination spans. The finished engine was compared with docmv, markmv, and mdref in [Markdown options](../Investigation/markdown_Options.md). `ignore` 0.4 finds the Markdown files while honouring `.gitignore` (it is the crate behind ripgrep), and `percent-encoding` 2 decodes `%20` in destinations; `percent-encoding` was already in the tree through `url`, while `ignore` is new (with `globset` and `bstr`).
