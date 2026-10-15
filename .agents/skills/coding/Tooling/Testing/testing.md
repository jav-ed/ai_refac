# Testing

Tests cost real time: writing, running, waiting, fixing, maintaining. When 95% of the time goes into handling tests, we are not making progress.

AI agents love to create tests because they believe tests help. Managing and running them is the pain described above. So:

1. Before creating a test, justify it. Cannot justify it? Do not write it.
2. Can you? Rate its benefit from 1 to 10. If it is not above 6, do not write it.
3. For every test that stays, ask whether a cache can speed it up. See [Moon](Moon/moon.md): which tests a cache can serve, how to make a task cacheable, and how to run only what changed.
4. Check the project's `AGENTS.md`: it can override this section, mostly with stricter rules.

## Tools, for a test that is justified

- [Moon](Moon/moon.md): caches and scopes test runs and checks; which languages it fits and how to make a task cacheable.
- [Maestro](Maestro/maestro.md): UI automation and end-to-end tests for mobile apps, Android in particular; promising, not yet investigated.

## Language-specific testing

- [JS / TS testing](../../Languages/Js_Ts/testing.md): choosing Bun or Vitest, suffix per runner, grouping tests by cost, time budgets, real-database tests, and which tests are not worth writing.
