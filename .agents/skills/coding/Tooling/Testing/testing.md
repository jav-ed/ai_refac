# Testing

Tests cost real time: writing, running, waiting, fixing, maintaining. When 95% of the time goes into handling tests, we are not making progress.

AI agents love to create tests because they believe tests help. Managing and running them is the pain described above. So:

1. Before creating a test, justify it. Cannot justify it? Do not write it.
2. For every test that stays, ask whether its result can be reused safely. Prefer the build tool's native cache before adding an outer task runner; use the choice below.
3. Rate its benefit from 1 to 10, against a bar that scales with what the test costs. **Default:** a cacheable test earns the lowest bar, 4 and above, since it replays free once nothing relevant changed; an uncached test pays its real cost every run, forever, so the bar is higher, and not above 6 means no test. A repository whose own uncached tests vary a lot in cost (a quick check against one that loads a whole schema, say) can split that bar further and record its own numbers.
4. A repository's own numbers, if any, live under its own `Tooling/` docs folder — check `Tooling/linker_Tooling.md` first. Not there? Nothing is overridden; the default above applies.

## Choose the cache owner

**Default:** use Gradle's native cache for Gradle projects. Add Moon or Nx when the repository needs orchestration across tools or languages, or measured CI features beyond the native workflow. Keep an existing, working repository entry point unless there is a reason to change it.

| Situation | Starting point |
| --- | --- |
| Java, Kotlin, Android or KMP work owned by Gradle | Native Gradle task reuse and build cache; configuration caching separately speeds build setup. Verify the actual task and platform. |
| Repeated checks without adequate native reuse, or an existing shared Moon workflow | Moon, with complete task inputs and outputs. A Gradle command can be wrapped as a system task. |
| Rust: Cargo caches compiling and linting but never a test result | Run natively first. Moon pays only for slow deterministic suites, several crates or a chain into other languages; it did not for a suite that runs in 0.3 s. See [Rust testing](../../Languages/Rust/testing.md). |
| A workspace needs Nx's project graph, affected selection or broader CI orchestration | Evaluate Nx; its `@nx/gradle` integration can infer Gradle projects and tasks. Verify the installed versions and actual task execution. |

An outer cache cannot repair missing native inputs, and inferred tasks do not prove complete input tracking. Evaluate an added layer against the native baseline: real execution, unchanged reuse, output restoration, input/environment invalidation, failure propagation, wall time and memory. A cached or affected-skipped result is not fresh test execution. Remote sharing alone does not require an outer runner: Gradle supports a remote build cache too.

- [Gradle](Gradle/gradle.md): enable native caching, run focused or fresh tests, declare external inputs, verify reuse and invalidation, and maintain local or CI caches. Use for Gradle-owned JVM tests; check other target task types separately.
- [Moon](Moon/moon.md): configure caching and task orchestration when native reuse is insufficient or the repository already uses Moon; includes the additional contract when wrapping Gradle.
- [Nx Gradle integration](https://nx.dev/docs/technologies/java/gradle/introduction): official setup and task-inference documentation; consult when Nx's graph or CI orchestration is needed, rather than treating Nx as an automatic replacement for Moon.

## Other tools, for a test that is justified

- [Maestro](Maestro/maestro.md): UI automation and end-to-end tests for mobile apps, Android in particular; promising, not yet investigated.

## Language-specific testing

- [JS / TS testing](../../Languages/Js_Ts/testing.md): choosing Bun or Vitest, suffix per runner, grouping tests by cost, time budgets, real-database tests, and which tests are not worth writing.
- [Rust testing](../../Languages/Rust/testing.md): Cargo never caches a test result, `cargo test` against nextest (and the shared-file race nextest exposes), rating Rust tests, and whether Moon pays for a crate or workspace; the Moon setup itself is [Moon for Rust](../../Languages/Rust/moon_Cache.md).
