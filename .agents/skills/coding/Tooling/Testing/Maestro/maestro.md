# Maestro: mobile UI automation and tests

[Maestro](https://github.com/mobile-dev-inc/maestro) is an open-source (Apache 2.0) framework for UI automation and end-to-end tests of mobile and web apps: Android, iOS, and web, including React Native and Flutter apps. A test or automation is a YAML flow, a list of steps such as `launchApp`, `tapOn`, and `assertVisible`. Flows run on emulators, simulators, browsers, and physical Android devices (physical iOS devices are not supported yet). The CLI is free; Maestro Cloud, its hosted way to run flows in parallel, is paid.

Status: **promising for Android, not yet investigated by us.** Maestro is a separate thing from Moon. Moon decides whether a command has to run again; Maestro is what a command can run: scripted interaction with a real app. That makes it useful for tests and for automation alike, for example a repeatable walk through the app's main screens.

## Before relying on it

Run our own investigation and document the result in the project's docs. Cover at least:

- How it runs in our setup: Java version, emulator or device, how a flow is started from the command line.
- How long a flow takes, and how often it fails for reasons that are not bugs (timing, emulator state).
- What a flow costs to maintain when the screens change.
- Whether the free CLI is enough or only the paid Cloud scales to what we need.

## How it fits with the testing rules

- A flow needs a device or an emulator, which is state outside the files, so it cannot be cached. If it runs through Moon, it is an uncached task that runs before a release, not after every change (see [Moon](../Moon/moon.md)).
- A UI flow is a slow, high-maintenance kind of test. It must pass the justification and benefit score in the [Testing rules](../testing.md) like any other test: protect a rule that nothing cheaper protects.

## When you need the Maestro docs

Read them locally instead of guessing commands. The `doc-start` skill explains how to clone a repository shallowly into the project's `Repos/` folder and record it in `Project_Manag/Docs/Setup/repos_List.md`. For Maestro that is `mobile-dev-inc/maestro`. Check `repos_List.md` first, it may already list a clone.
