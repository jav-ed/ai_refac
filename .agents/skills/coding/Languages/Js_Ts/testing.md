# JS / TS testing

What we learned from running JS and TS tests at scale: which runner to pick, how to group tests by cost, and which limits to enforce. Read it after a test is justified (see [Testing](../../Tooling/Testing/testing.md)); to cache a test, go on to [Moon](../../Tooling/Testing/Moon/moon.md). The numbers come from one real project and show the size of an effect, not what your machine will do.

## Pick the runner by what the test needs

- **Default to Bun's test runner (`bun:test`)** for anything that needs no DOM and no build-tool plugin. It is much faster: 374 plain unit files took 16 s with Bun in parallel and 65 s in Vitest.
- **Use Vitest only where Bun cannot:** tests that need the bundler's pipeline (a framework compiler, plugins, `?raw` imports), tests that use `vi.` mocking or fake timers, and tests that need a DOM (jsdom).
- **A file imports one framework, never both.** Bun's test API is not a drop-in for Vitest's, so move a file to Bun only when it passes there unchanged.

## Let the file suffix name the runner

Give each runner its own suffix, so a new test is found by its name and nobody edits a list: `*.test.ts` (Bun), `*.vite-test.ts` (Vitest in Node), `*.ui-test.tsx` (Vitest with jsdom), `*.integration_test.ts` (real database), `*.flow.ts` (HTTP flows against a running server). Without this, a test file can sit in the tree and run nowhere, which is worse than no test.

- Keep one cheap guard that fails when a test-like file has no known suffix, imports another runner's framework, or when a runner's configuration can no longer find its files (a narrowing `--dir` flag, for example).
- Put the exclusion of vendored and scratch folders (reference clones, disposable copies) into every runner's own configuration, not into a command-line flag. A plain `vitest run` or `bun test` from the root must never collect tests that are not yours.

## Group tests by cost, and never mix groups in one command

1. **Cached and cheap:** unit tests, UI tests, the type check, and the style check. A cache replays them instantly while nothing changed.
2. **Integration, light:** tests against a real database. Never cached, about a minute.
3. **Integration, heavy:** files whose setup loads a large schema before the first test. They get their own command and a banner that says how long to expect. Heavy means the setup, not the tests.
4. **Flow tests** (HTTP against a running server) stay separate: uncached and outside the quality gate, because the server's state changes outside the repository.

Database, clock, and shared-server tests are not cached: their result is not decided by the files alone, and almost any edit invalidates them anyway.

## Enforce budgets, do not request them

No single test or hook waits longer than 15 seconds, no file longer than 90 seconds, and a group over 5 minutes is reported. A test that truly needs more is a named exception with its reason and duration (a heavy setup, for example). Enforce it where it cannot be ignored: the runner's timeout flag and a guard that rejects larger explicit timeouts.

## Know where the time goes

Module loading dominates, not the assertions. In one project, 547 unit files spent 39 s in tests and the rest of a 3m40s run loading modules; for UI tests, jsdom setup per file and module loading together took more than the tests. So lighter imports and fewer files help more than faster assertions, and scoping a run to the files a change can affect (Vitest's `--changed` runs only the tests that import a changed file) helps most.

## Tests against a real database

- Use the same storage engine as production. In one project an in-memory engine made 17 tests fail with "failed transaction" that pass on the real one.
- Each file creates its own uniquely named database and removes it afterwards. Connect through one guarded helper that accepts only the disposable instance and fails within seconds, with a message that says what is missing.
- Run few files at a time (four was the ceiling for one database), and never kill a run halfway: a setup that times out leaks databases, and a wedged database needs a reset.

## Tests that are not worth writing

- Tests of fictional development data: seed generators and the shape of fixtures. A broken seed shows the moment you seed.
- Tests of decoration: animations, confetti, icon motion, skeleton shapes.
- Tests of developer-machine plumbing: container supervision, seed-cache mechanics.
- One-time migration tests. They pin the schema they started from and break at the next schema change; rehearse the release on a restore of the real data instead.
- Tests that restate what another suite already proves.
- Any test that calls a paid or live external service. Use an in-memory fake that fails on any unexpected operation, and keep at most one sandbox test, run on purpose.
