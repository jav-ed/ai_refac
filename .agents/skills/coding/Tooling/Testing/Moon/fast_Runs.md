# Fast and safe Moon runs

Rules for running Moon tasks without wasting time or trusting a wrong result. Read it when a run is slow, looks wrong, backs a required gate, or when you are about to measure how much the cache saves. The cached-or-not decision and the everyday commands are in [Moon](moon.md); defining a task is in [Make a task cacheable](task_Setup.md).

## Read results correctly

- A cached run replays old output, including the old test duration. Judge by Moon's final status (cached or executed), not by the duration the test runner printed, and use `moon run --force <project>:<task>` when a fresh run is needed.
- A different set of arguments is a different cache entry. A passing scoped run says nothing about the full suite.

## Measure honestly

- Measure on frozen inputs. If a file changes while you measure, the second run is not a warm-cache run: it reruns, and the numbers say nothing about the cache.
- Compare the task hashes of the two runs first (`.moon/cache/hashes/`). Equal hashes mean the second run could replay; different hashes mean something changed, and the manifests show what.
- Individual timings are single observations. Report what was measured and do not extrapolate it to other tasks.

## Choose tasks and scope with care

- `--affected` and `moon ci` pick tasks by changed files only. They can skip a task whose result depends on the environment or on a gitignored file, so do not use them for a required gate that depends on those.
- More tasks add startup and scheduling work. Split a suite into several tasks only after measuring a gain.

## Keep it fast

- Put a hard time limit on every test and hook. A test that waits out a timeout is a defect: a missing prerequisite fails within seconds with a message that says what is missing and how to fix it.
- Start long runs in the background and read the result when they end. Do not poll in a loop.
- A failed Moon command is an error to report. Do not build a wrapper script, loop, or process kill around it.
- Keep Moon the single entry point: a package script or recipe may call `moon run`, and Moon must never call a script that calls Moon again.

For a task option not covered here, read the current Moon docs locally: see [When you need the Moon docs](moon.md#when-you-need-the-moon-docs).
