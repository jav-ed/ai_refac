use crate::drivers::resolve_resource_path;
use crate::drivers::typescript::dependencies;
use anyhow::{Context, Result, bail};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::process::Command;

/// Folder name of the per-platform native TypeScript 7 package, which ships
/// the compiler and language server as one executable.
fn platform_package() -> Result<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => Ok("typescript-linux-x64"),
        ("linux", "aarch64") => Ok("typescript-linux-arm64"),
        ("macos", "x86_64") => Ok("typescript-darwin-x64"),
        ("macos", "aarch64") => Ok("typescript-darwin-arm64"),
        (os, arch) => bail!("The TypeScript native engine is unsupported on {os}/{arch}"),
    }
}

/// Install (when needed) and return the TypeScript 7 native executable that the
/// helper's locked `typescript-native` dependency provides. It runs beside the
/// TypeScript 6 helper because TypeScript 7 removed the JavaScript API that
/// file moves use.
pub async fn locate() -> Result<PathBuf> {
    let script_path = resolve_resource_path("scripts/ts_refactor.ts")?;
    let script_dir = script_path
        .parent()
        .context("Could not determine scripts directory")?;
    dependencies::ensure_installed(
        &dependencies::bun_command(),
        script_dir,
        &["typescript-native"],
    )
    .await?;
    let executable = script_dir
        .join("node_modules/@typescript")
        .join(platform_package()?)
        .join("lib/tsc");
    if !executable.exists() {
        bail!(
            "TypeScript native executable missing: {}. Run `bun install --frozen-lockfile` in {}",
            executable.display(),
            script_dir.display()
        );
    }
    Ok(executable)
}

/// The engine silently ignores some options TypeScript 7 removed (`baseUrl`
/// breaks import resolution without any error), which would turn a rename into
/// a partial one. Its own project loader reports those as config errors, so a
/// failed file listing is the authoritative guard: hard-fail instead of
/// guessing. The listing must also contain the target, otherwise the engine
/// would rename inside a different tsconfig than the one the caller named.
/// A batch checks every file it renames in, with the one listing.
///
/// `astro` lets the engine run the project's content mappers, which is what
/// puts `.astro` files in the listing. A tsconfig that lists mappers is refused
/// without it, and the refusal says how to allow them.
pub async fn preflight(
    executable: &Path,
    project_root: &Path,
    targets: &[PathBuf],
    timeout: Duration,
    astro: bool,
) -> Result<()> {
    let mut command = Command::new(executable);
    command.args(["-p", ".", "--listFilesOnly"]);
    if astro {
        command.arg("--runExternalCode");
    }
    let output = tokio::time::timeout(
        timeout,
        command
            .current_dir(project_root)
            .kill_on_drop(true)
            .output(),
    )
    .await
    .context("The TypeScript engine timed out while loading the project")?
    .context("Could not run the TypeScript native engine")?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !output.status.success() {
        let errors: Vec<&str> = stdout
            .lines()
            .filter(|line| line.contains("error TS"))
            .collect();
        if errors.iter().any(|line| line.contains("TS18068")) {
            bail!(
                "This project's tsconfig.json lists `contentMappers`, which run code from the project's node_modules (for example @astrojs/ts-content-mapper) so that .astro files are part of the program. Add --astro (or REFAC_ASTRO=1) to allow that. Nothing was changed."
            );
        }
        bail!(
            "The TypeScript 7 native engine rejects this project's tsconfig:\n{}\nSymbol rename needs a configuration TypeScript 7 accepts. Nothing was changed.",
            if errors.is_empty() {
                stdout.trim().to_string()
            } else {
                errors.join("\n")
            }
        );
    }
    if astro {
        require_node()?;
        require_astro_files(project_root, &stdout)?;
    }
    for target in targets {
        let name = target.file_name();
        let listed = stdout
            .lines()
            .map(Path::new)
            .filter(|path| path.file_name() == name)
            .any(|path| {
                path == target
                    || path
                        .canonicalize()
                        .is_ok_and(|canonical| canonical == *target)
            });
        if !listed {
            bail!(
                "{} is not part of the tsconfig in {}. Point --project-path at the package whose tsconfig includes the file.",
                target.display(),
                project_root.display()
            );
        }
    }
    Ok(())
}

/// `--astro` renames inside `.astro` files only if the engine reads them. A
/// project that has such files and lists none of them has no content mapper
/// for them, and the rename would silently skip every one.
fn require_astro_files(project_root: &Path, listing: &str) -> Result<()> {
    let has_components =
        !crate::drivers::typescript::files::find(project_root, &["astro"])?.is_empty();
    if has_components && !listing.lines().any(|line| line.ends_with(".astro")) {
        bail!(
            "--astro was given, but the TypeScript engine lists no .astro file of {}. Install @astrojs/ts-content-mapper in the project and list it in tsconfig.json:\n  \"contentMappers\": [{{ \"package\": \"@astrojs/ts-content-mapper\", \"extensions\": [\".astro\"] }}]\nNothing was changed.",
            project_root.display()
        );
    }
    Ok(())
}

/// The Astro content mapper is a Node.js program that the engine starts.
fn require_node() -> Result<()> {
    which::which("node").map(|_| ()).map_err(|_| {
        anyhow::anyhow!(
            "--astro needs Node.js on the PATH: the TypeScript engine starts the Astro content mapper (@astrojs/ts-content-mapper) with `node`. Install Node.js and run again. Nothing was changed."
        )
    })
}
