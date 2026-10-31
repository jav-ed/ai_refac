# Gradle: cached tests, setup and maintenance

Gradle includes native task-output caching. **Default:** use it for Gradle-owned work before adding Moon or Nx. The ordinary test command handles reuse once caching is enabled; no separate cached edition or extra plugin is needed. Tool selection belongs in [Testing](../testing.md): when native reuse is sufficient and when an outer runner adds value.

This guide covers standard JVM `Test` tasks, including those created by Kotlin plugins. Android device tests and Kotlin/Native or iOS tests use other task types and environments: check their plugin documentation and prove their behavior separately. A cached JVM result does not prove a current device check.

## Set up

1. Use the repository's Gradle Wrapper (`./gradlew`, or `gradlew.bat` on Windows) and its pinned version. Inspect existing properties, task configuration and repository instructions before changing them. Find task names with `./gradlew tasks --all`; inspect one with `./gradlew help --task :my-service:test`. Replace the example module, task and class names throughout this page with real ones.
2. Enable eligible task-output caching in the root `gradle.properties`:

   ```properties
   org.gradle.caching=true
   ```

   For one invocation only, use `--build-cache`. The preconfigured local cache needs no server. Standard `Test` tasks support caching; arbitrary custom tasks need explicit opt-in and complete inputs/outputs. Do not mark every task cacheable indiscriminately.
3. Try a representative command with `--configuration-cache`. After confirming compatibility, separately enable build-setup reuse:

   ```properties
   org.gradle.configuration-cache=true
   ```

   Keep configuration-cache problems as errors. Diagnose the generated problem report and fix incompatible build logic; use an explicit, documented `--no-configuration-cache` invocation when needed. Do not leave `problems=warn` as the permanent setup. Task-output caching can remain enabled independently.

## Everyday use

Run from the Gradle build root. JVM `Test` tasks support class or method filters:

```bash
./gradlew :my-service:test
./gradlew :my-service:test --tests 'example.ClockTest'
./gradlew :my-service:test --rerun
./gradlew :my-service:test --info
```

These mean normal reuse, a focused run, fresh execution of this test task, and diagnosis. `--rerun` is a task option: it does not force every prerequisite to execute. Check the pinned wrapper's help for availability. Broad `--rerun-tasks` forces prerequisites too; reserve it for cases that need that work. `--no-build-cache` alone still permits up-to-date skipping, so it does not guarantee fresh tests.

| Observation | Evidence |
| --- | --- |
| `UP-TO-DATE` | Current inputs and existing outputs match; test execution was skipped. This reuse does not require the build cache. |
| `FROM-CACHE` | Matching task outputs were restored; tests did not execute. |
| Configuration cache reused | Build setup was restored; this says nothing about whether tests executed. |
| Task executes and fresh reports contain the intended tests | Fresh execution evidence for that selected scope. |
| `SKIPPED`, `NO-SOURCE`, or zero selected tests | No proof that the intended tests passed. |

Read the task outcomes and actual reports, not only a successful build heading or old durations in restored XML. Record selection and whether results were reused or freshly executed. Caching is per task, not per assertion: a shared classpath change can invalidate an entire selected suite. Splitting tasks by cost helps choose what to run, but does not automatically prove which consumers a source change affects.

## Keep inputs complete

Gradle's standard test task models test classes, dependencies and configured test properties. It does not discover every file, live service or environment value read by test code. A filename passed as a system property tracks that string, not automatically the file's contents.

For an extra fixture, add its actual bytes as an input in the test-owning module's `build.gradle.kts`. Put imports at the top and task configuration after the existing plugin setup. Adapt the task name; KMP projects often use a name such as `jvmTest`:

```kotlin
import org.gradle.api.tasks.PathSensitivity
import org.gradle.api.tasks.testing.Test

val scenario = layout.projectDirectory.file("test-data/scenario.json")

tasks.named<Test>("test") {
    // The test opens this file directly, outside its compiled resources.
    inputs.file(scenario)
        .withPropertyName("scenario")
        .withPathSensitivity(PathSensitivity.RELATIVE)
    systemProperty("test.scenario", scenario.asFile.absolutePath)
}
```

Declare relevant values with `inputs.property` and files with `inputs.file`, `inputs.files` or `inputs.dir`, using Gradle providers for environment/property values. Track only what changes the result. Absolute path strings can reduce reuse across checkouts even when file path sensitivity is relative. A manifest alone cannot detect unrecorded changes to the files it describes. Separate expensive asset-integrity checks so ordinary tests do not hash a huge unrelated data tree.

For checks of live state that cannot be modeled, keep execution fresh. On an existing dedicated `Test` task, for example:

```kotlin
tasks.named<Test>("integrationTest") {
    // This task observes the live service on every invocation.
    outputs.upToDateWhen { false }
    outputs.doNotCacheIf("Checks live service state") { true }
}
```

The two controls explicitly address up-to-date reuse and output caching. Create that task through the project's existing test setup before configuring it. Apply the restriction to the live checks, not every test. Controlled clocks, seeded randomness and isolated fixtures can make other tests reproducible. Side-effecting operations such as deployments must execute when requested.

## Verify the contract after changing it

Use a small representative task and synthetic fixtures in a throwaway workspace. Freeze unrelated inputs, record wrapper/JDK/plugin versions and exact commands, and keep heavy runs sequential on memory-constrained machines.

1. Execute the selected tests with caching enabled and confirm their count and reports. Repeat unchanged; expect `UP-TO-DATE` unless the task deliberately requires fresh execution.
2. Once outputs have been stored in the build cache, remove only that task's generated outputs in the throwaway workspace. Repeat and verify `FROM-CACHE` plus restored reports. Do not delete source data or the cache being tested.
3. Change a meaningful fixture to deliberately violate an assertion. Verify actual task execution and the expected failure. An unchanged run alone cannot prove invalidation works. A synthetic stand-in can test this boundary without touching protected project data.
4. Restore the fixture; matching passing outputs may be restored or execute again. Change an unrelated document and check for unnecessary invalidation. Check relevant environment inputs separately when present.

If an outer runner is involved, also verify its output restoration and failure propagation with independent invocation evidence. For fresh execution, bypass its cache **and** request Gradle's `--rerun`; forcing only the outer command can still leave Gradle reusing results. An outer hit can skip Gradle entirely, so its inputs must cover the complete inner task contract, including relevant toolchain/runtime values and external data.

## Maintain and diagnose

- **When dependencies change:** review new file/environment reads and custom task outputs. Repeat the small invalidation/restoration check after changing that contract or upgrading build plugins. Keep project commands and measurements in that project's docs; this skill stays reusable.
- **On a suspicious pass:** rerun the selected task fresh, inspect reports and `--info` reasons, then fix missing inputs. Clearing caches alone does not repair the contract. For a miss, compare actual inputs, filters, runtime settings and toolchain before broadening globs.
- **Storage:** the default local build cache is under the Gradle User Home (`~/.gradle/caches`); configuration state is under the build's `.gradle/configuration-cache`. These are generated disk data, excluded from source control. Keep the Wrapper and source configuration tracked. Gradle periodically expires unused entries; avoid routine cache deletion or `clean` in the edit/test loop.
- **Memory:** disk cache size is separate from daemon/test-worker memory. Reuse does not require extra parallel forks or large heaps. Choose worker and heap limits from the machine's budget; do not copy another project's numbers. Do not stop other sessions' daemons as routine cache maintenance.
- **CI and sharing:** an ephemeral runner needs persisted cache storage or a separately configured remote cache to benefit across jobs. Gradle already supports remote output caching; it is optional. Set trusted write permissions, credentials and retention through the CI's supported workflow, and verify restoration. Neither a shared cache nor an outer runner fixes an invalidated slow test suite.

## Official references

Checked 2026-10-21. Consult documentation for the repository's pinned Gradle/plugin versions when adapting examples.

- [Build cache](https://docs.gradle.org/current/userguide/build_cache.html): enabling local reuse, eligible tasks and optional remote output caching.
- [Configuration cache setup](https://docs.gradle.org/current/userguide/configuration_cache_enabling.html): independent enablement, compatibility failures, reports and explicit opt-outs.
- [Task inputs and outputs](https://docs.gradle.org/current/userguide/incremental_build.html): file/value declarations, providers and correct up-to-date behavior.
- [JVM Test task](https://docs.gradle.org/current/dsl/org.gradle.api.tasks.testing.Test.html): task configuration, filters and report properties.
- [Command-line options](https://docs.gradle.org/current/userguide/command_line_interface.html): scoped commands, task-specific `--rerun` and broad `--rerun-tasks`.
- [Output reuse controls](https://docs.gradle.org/current/javadoc/org/gradle/api/tasks/TaskOutputs.html): `upToDateWhen` and `doNotCacheIf` for checks that observe live state.
- [Directories and cleanup](https://docs.gradle.org/current/userguide/directory_layout.html): cache locations, automatic retention and configuration options.
