# Application

Concrete application repo layer. Use this when adding CI/CD to one Astro or compiled Rust repo, checking that repo's trigger path, or running an already-integrated pipeline.

## Routing

- [Active use](Active_Use/daily_Workflow.md): normal push, manual preflight, deploy, rollback, verification, and debugging after the repo already has working CI/CD files.
- [Read a pipeline run's output](Active_Use/read_Pipeline_Output.md): list recent Woodpecker pipelines, see which step failed, and print a step's log with the GET-only helper. Read it before guessing why a push did not build or deploy.
- [Integration](Integration/first_Setup.md): first setup for bringing one Astro repo into the existing platform; copy the template, complete `.woodpecker.yml` and `Code/CI_CD/Config/env.env`, verify scripts locally, then route to trigger checks.
- [Configure variables](Integration/configure_Variables.md): practical variable map for integration. Use this when setting `CI_REPO_PATH` in `.woodpecker.yml` and project/deploy values in `Code/CI_CD/Config/env.env`.
- [Rust binary tool](Integration/rust_Binary_Tool.md): tested compiled-tool integration with explicit Rustup/Cargo variables, isolated-HOME validation, modular Just recipes, atomic installation, and first-trigger verification.
- [IndexNow notifications](Integration/indexNow_Notifications.md): operator path for the opt-in Astro integration; it routes implementation changes and deep invariant diagnosis to `Integration/Detail/`.
- [Local backend workspace decision](Integration/Detail/local_Backend_Workspace_Decision.md): background context for why `skip_clone: true` still starts in a temporary workspace and why `CI_REPO_PATH` lives in `.woodpecker.yml`. Read only when debugging local-backend behavior or changing the CI/CD design.
- [Trigger checks](Trigger/repo_Trigger_Checks.md): project-side `.woodpecker.yml`, `justfile`, `Code/CI_CD/`, Forgejo webhook, and Woodpecker repo state checks.
- [Trigger setup](Trigger/repo_Trigger_Setup.md): API-backed setup or repair for one repo's Woodpecker activation and Forgejo webhook.

Shared Forgejo/Woodpecker service checks, API discovery, and secrets live under [Platform](../Platform/linker_Platform.md).
