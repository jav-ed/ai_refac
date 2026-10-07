# Investigation

In-depth studies that decided how a feature is built, kept here because Scratch is disposable and the reasoning must outlive it. Each document records what was tried, on which cases, with which numbers, and which failure modes surfaced. Open one before changing the engine it covers or before adding the same capability for another language.

## Docs

- [TypeScript rename engines](typescript_Rename_Engines.md): which tools can rename a TypeScript symbol (TypeScript 7 native, TypeScript 6 LanguageService, typescript-language-server, Biome, oxlint, Oxc, ts-morph), how each did on 16 cases, speed and memory on a 2,100-file project, and the silent failures found: undetected name clashes and shadowing, and `baseUrl` making the TypeScript 7 engine miss files without an error.
- [Kotlin options](kotlin_Options.md): which tools could give Kotlin file and package moves and symbol rename for Android work (JetBrains Kotlin language server, fwcd server, OpenRewrite, IDE plugins, Analysis API), why the official server is the recommended engine, and the hands-on tests still pending.
