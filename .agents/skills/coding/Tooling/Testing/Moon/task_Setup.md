# Make a Moon task cacheable

How to define a task so its cached result can be trusted. Read it when adding or changing a task, when a cached pass looks stale, or when a task reruns for no visible reason. For the cached-or-not decision and the everyday commands, go back to [Moon](moon.md).

```yaml
# moon.yml
tasks:
  test:
    command: cargo test
    inputs:
      - src/**/*
      - Cargo.toml
      - Cargo.lock
    outputs:
      - target/test-report.xml
  migrate-check:
    command: ./check_migrations.sh
    options:
      cache: false
```

## Inputs

- List every input that can change the result: sources, config files, lockfiles, patches, test data, and the environment values the test reads (`$VAR_NAME` is a valid input).
- A cached pass is only as trustworthy as its inputs. A missing input replays a stale pass, which is worse than a slow run. When a test reads a file the task does not list, add it.
- Prefer explicit inputs over a greedy glob. Moon's default `**/*` pulls in everything under the task's folder, and files can still slip in or out in ways you do not expect.
- Glob inputs skip gitignored files. If the result depends on one (`.env`, generated sources), check that Moon really hashes it, and hash it in a step of your own if it does not.
- Keep the inputs when narrowing a task's scope: a test may read files outside its own folder, such as schemas, fixtures, or generated code.

## Environment

List only the variables the test reads. If a project hashes the whole environment instead, it must exclude shell bookkeeping such as the shell depth (`SHLVL`): that value differs between equivalent ways of launching the same command, and in one project it caused a cache miss on every second launch. Compare the hashes of two equivalent runs to find such a variable.

## Outputs

Declare every file the task produces as an `outputs` entry, so a cache hit restores it. A task whose result is only its exit status needs none.

## What never gets cached

- Anything with side effects or outside state: database writes, migrations, deployments, tests that read a database, the clock, or the network. Set `options: cache: false` and say why in a comment next to it.
- Release and production builds. An input exclusion can hide a change, such as an ignored generated image, and a stale artifact would then ship. Build releases fresh every time.

## Check what a task hashes

Moon writes a manifest for every hash to `.moon/cache/hashes/<hash>.json`, listing every source that went into it. Read it when a task reruns unexpectedly or replays a stale result, and audit it routinely for tasks with broad globs. Keep `.moon/cache` out of version control.

After changing a task, check three things once: an unchanged rerun replays, a change to a listed input reruns, and a change to something unlisted does not rerun (then ask whether it should have). `moon task <project>:<task> --json` shows the task as Moon sees it.

For a task option not covered here, read the current Moon docs locally: see [When you need the Moon docs](moon.md#when-you-need-the-moon-docs).
