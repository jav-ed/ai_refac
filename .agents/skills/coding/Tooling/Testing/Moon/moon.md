# Moon: cache and scope tests and checks

Moon runs tasks in dependency order and caches each task's result by hashing its inputs. When nothing a task depends on has changed, the next run replays the result instantly instead of executing the command. A task is any command, so any language can run in it, but how much help Moon gives differs by language (see [Which languages fit](#which-languages-fit)). Moon does not replace your test runner (`pytest`, `cargo test`, `vitest`): it decides whether the runner has to run at all.

Read this after a test is justified (see [Testing](../testing.md)). The question for every test that stays is: can a cache make it free?

## When to use it

Use Moon when a test or check runs more than once and its result depends only on files: type checks, linters, formatters, unit tests. The second run is free while nothing changed. Skip it for a one-off command, and for work with side effects such as deployments or migrations, which must never replay from a cache.

## Which languages fit

Caching works for any command. What differs is how deeply Moon understands the language (its docs call these tiers):

- **Deep support: JavaScript and TypeScript (Bun, Deno, Node with npm, pnpm, or yarn), Go, and Rust (Cargo).** Moon reads manifests and lockfiles to infer dependencies and tasks, hashes dependency changes for you, and downloads the pinned toolchain version. JavaScript is Moon's home ground.
- **Partial or experimental: Python (pip, Poetry, uv) and Ruby (gems, Bundler).** Project categorization works; the ecosystem integration is still marked experimental. PHP and Bash have project categorization only.
- **Everything else (C# and so on):** Moon runs the command as a plain `system` task. The cache still works, but Moon infers nothing and installs nothing, so every input is yours to list.
- **Java, Kotlin, and Android (Gradle): not tested by us yet.** Moon's docs list them at the lowest tier only, so they would run as plain `system` tasks. We do not know how well that works, and nobody should assume it does. See [Java, Kotlin, and Android](#java-kotlin-and-android).

Moon adds the most where the language's own tool reruns everything on every call: test runners such as `pytest`, `cargo test`, and `vitest`, linters, type checkers, and formatters. It adds little where the native tool already skips unchanged work: `go test` caches its results, and Gradle (Android, JVM) skips up-to-date tasks and has its own build cache. There, use Moon only as one shared entry point, or not at all, rather than keeping two caches correct.

## Java, Kotlin, and Android

Moon's support for Java and Kotlin is untested here, and Android projects (Gradle) are the case that matters. Treat Moon as unproven there until we have run our own experiment and documented the result in the project's docs. Gradle already skips up-to-date tasks and has its own build cache, so Moon must add something on top to be worth keeping.

The experiment: in a throwaway copy of an Android or Gradle project, wrap the test and lint commands in Moon tasks, and record what happens. The checks that matter, each proven by a log of which commands actually ran (replayed console output proves nothing):

- An identical repeat does not run the command.
- Deleting the declared outputs restores them without running the command.
- A change to an unrelated file does not rerun the task, and a change to a listed input does.
- A change to a declared environment value reruns the task.
- A failing task stops the tasks that depend on it.
- The result is not slower or less correct than Gradle's own caching alone.

Write down the tool versions, the commands, what passed, what failed, and the timings, with the limits of the measurement. If Moon does not work well here, the fallback is Nx, which is reported to support Java, Kotlin, and Gradle through its Gradle plugin. Test Nx with the same checks instead of trusting that claim.

For UI automation and tests of an Android app on a device or emulator, see [Maestro](../Maestro/maestro.md): a separate tool, not part of Moon's caching.

## Can this test be cached?

- **Yes** when the result depends only on files and the pinned toolchain: unit tests, UI tests on a fake DOM, type checks, linters, formatters. Put it in a cached task.
- **No** when it depends on state outside the files: a database, the clock, the network, randomness. Mark the task uncached, say why in a comment, keep the test light, and run it only before a release, not after every change.

## Everyday commands

```bash
moon run app:test                           # goes through the cache; unchanged inputs replay instantly
moon run app:test -- path/to/one_test_file  # only what you changed; other arguments are another cache entry
moon run --force app:test                   # a deliberate fresh run
moon task app:test --json                   # shows what the task hashes
```

Moon caches per task, not per assertion: any relevant source edit invalidates the whole task, and the next full run is a full run. So scope the run to the files the change can affect instead of running the whole suite after a small edit.

## When you need the Moon docs

Task options and commands change between Moon versions, so do not guess them from memory. Read the current docs locally. The `doc-start` skill explains how to clone a repository shallowly into the project's `Repos/` folder (gitignored) and record it in `Project_Manag/Docs/Setup/repos_List.md`, so the latest docs are readable without network calls. For Moon that is the `moonrepo/moon` repository; its docs source is under `website/docs`, and the language support table is in `website/docs/intro.mdx`. Check `repos_List.md` first, it may already list a Moon clone and say where it lives under `Repos/`. If not, run `git clone --depth 1 https://github.com/moonrepo/moon Repos/moon` and add the entry. Compare the clone's version with `moon --version`.

## Navigation

- [Make a task cacheable](task_Setup.md): the `moon.yml` shape and the rules for inputs (explicit versus greedy globs, gitignored files), environment variables and shell-depth misses, outputs, tasks that are never cached (side effects, release builds), and how to read what a task hashes. Read it when adding or changing a Moon task, when a cached pass looks stale, or when a task reruns for no visible reason.
- [Fast and safe runs](fast_Runs.md): why a cached run shows old timings, how to measure the cache honestly (frozen inputs, comparing hashes), why `--affected` and `moon ci` can skip a task, when splitting a task pays off, and the rules for time limits, background runs, and a failed Moon command. Read it when a run is slow, looks wrong, backs a required gate, or before you measure what the cache saves.
