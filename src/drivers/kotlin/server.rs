//! Locating, starting and waiting for the JetBrains Kotlin language server.
//!
//! The server imports the Gradle build before it can answer. It signals that
//! with `intellij/workspaceImportState` (phase FINISHED) and then
//! `intellij/ready-for-test`; it keeps opening short "Indexing" progress
//! tokens forever, so those cannot tell readiness. Requests sent earlier are
//! answered `null`, which is why this module waits for the real signals and
//! never sleeps.

use crate::drivers::lsp_session::{LspSession, SessionConfig};
use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::time::Duration;
use tempfile::TempDir;

pub const SERVER_ENV: &str = "REFAC_KOTLIN_SERVER";
pub const TIMEOUT_ENV: &str = "REFAC_KOTLIN_TIMEOUT_SECS";
const DEFAULT_TIMEOUT_SECS: u64 = 600;

/// The build the protocol above was verified against.
const VERIFIED_BUILD: &str = "263.6379.0";
const DOWNLOAD_URL: &str = "https://download.jetbrains.com/language-server/kotlin-server/263.6379.0/kotlin-server-263.6379.0.tar.gz";
const DOWNLOAD_SHA256: &str = "ab8ca4455dc2fc5fe1a24db2bccc46c104254d2c465155c4251ee65df8f3f7cc";

const KEPT_NOTIFICATIONS: &[&str] = &[
    "intellij/workspaceImportState",
    "intellij/ready-for-test",
    "intellij/importLog",
];

pub struct Install {
    dir: PathBuf,
    pub build: String,
}

impl Install {
    fn executable(&self) -> PathBuf {
        self.dir.join("bin").join("intellij-server")
    }
}

fn install_help() -> String {
    format!(
        "Download the Kotlin language server once and point refac at it:\n  curl -LO {DOWNLOAD_URL}\n  echo \"{DOWNLOAD_SHA256}  kotlin-server-{VERIFIED_BUILD}.tar.gz\" | sha256sum -c -\n  mkdir -p ~/.local/share/refac && tar -xzf kotlin-server-{VERIFIED_BUILD}.tar.gz -C ~/.local/share/refac\n  export {SERVER_ENV}=~/.local/share/refac/kotlin-server-{VERIFIED_BUILD}\nIt needs a JDK 17 or newer on PATH for the Gradle import."
    )
}

/// The installed server: the directory named by `REFAC_KOTLIN_SERVER`, which
/// must hold `bin/intellij-server` and `build.txt`.
pub fn locate() -> Result<Install> {
    locate_in(std::env::var_os(SERVER_ENV))
}

fn locate_in(configured: Option<OsString>) -> Result<Install> {
    let Some(value) = configured else {
        bail!(
            "{SERVER_ENV} is not set, so the Kotlin language server cannot be found.\n{}",
            install_help()
        );
    };
    let dir = PathBuf::from(value);
    let build_file = dir.join("build.txt");
    let build = std::fs::read_to_string(&build_file).with_context(|| {
        format!(
            "{SERVER_ENV} points to {}, which has no build.txt.\n{}",
            dir.display(),
            install_help()
        )
    })?;
    let Some(build) = build.trim().strip_prefix("ILS-") else {
        bail!(
            "{} does not name a Kotlin language server build (expected ILS-<number>)",
            build_file.display()
        );
    };
    let install = Install {
        dir,
        build: build.to_string(),
    };
    if !install.executable().is_file() {
        bail!("{} does not exist", install.executable().display());
    }
    Ok(install)
}

fn timeout() -> Result<Duration> {
    parse_timeout(std::env::var(TIMEOUT_ENV).ok().as_deref())
}

fn parse_timeout(configured: Option<&str>) -> Result<Duration> {
    let Some(value) = configured else {
        return Ok(Duration::from_secs(DEFAULT_TIMEOUT_SECS));
    };
    let seconds: u64 = value
        .parse()
        .ok()
        .filter(|seconds| *seconds > 0)
        .with_context(|| {
            format!("{TIMEOUT_ENV} must be a positive number of seconds, got `{value}`")
        })?;
    Ok(Duration::from_secs(seconds))
}

/// A running server with its throwaway system directory (about 250 MB of
/// caches that a second start does not reuse faster, so it is deleted).
pub struct KotlinServer {
    session: LspSession,
    pub build: String,
    timeout: Duration,
    system_dir: TempDir,
}

impl KotlinServer {
    /// Start the server on a Gradle root and return once the import is done.
    pub async fn start(install: &Install, project: &Path) -> Result<Self> {
        let timeout = timeout()?;
        let system_dir = tempfile::Builder::new().prefix("refac-kotlin-").tempdir()?;
        let (mut session, init) = LspSession::start(SessionConfig {
            name: "Kotlin language server",
            executable: &install.executable(),
            args: vec![
                "--stdio".to_string(),
                format!("--system-path={}", system_dir.path().display()),
            ],
            cwd: project,
            root: project,
            capabilities: client_capabilities(),
            keep_notifications: KEPT_NOTIFICATIONS,
            language_id,
        })
        .await?;
        check_capabilities(&init["capabilities"])?;
        session.initialized().await?;
        let build = install.build.clone();
        let server = Self {
            session,
            build,
            timeout,
            system_dir,
        };
        server.wait_until_ready().await
    }

    async fn wait_until_ready(mut self) -> Result<Self> {
        let import = self
            .session
            .wait_notification(
                |message| {
                    message["method"] == "intellij/workspaceImportState"
                        && message["params"]["phase"] == "FINISHED"
                },
                self.timeout,
            )
            .await;
        let import = match import {
            Ok(import) => import,
            Err(error) => return Err(self.fail(error).await),
        };
        if let Some(folder) = import["params"]["folders"]
            .as_array()
            .into_iter()
            .flatten()
            .find(|folder| folder["status"] != "SUCCESS")
        {
            let error = anyhow::anyhow!(
                "The Gradle import failed ({}) for {}",
                folder["status"],
                folder["folderUri"]
            );
            return Err(self.fail(error).await);
        }
        let ready = self
            .session
            .wait_notification(
                |message| message["method"] == "intellij/ready-for-test",
                self.timeout,
            )
            .await;
        match ready {
            Ok(_) => Ok(self),
            Err(error) => Err(self.fail(error).await),
        }
    }

    /// Stop the server and attach what the Gradle import printed.
    async fn fail(self, error: anyhow::Error) -> anyhow::Error {
        let import_log: Vec<String> = self
            .session
            .notifications()
            .iter()
            .filter(|message| message["method"] == "intellij/importLog")
            .filter_map(|message| message["params"]["message"].as_str())
            .map(|line| line.trim_end().to_string())
            // Stack frames bury the message that names the broken file.
            .filter(|line| !line.is_empty() && !line.trim_start().starts_with("at "))
            .collect();
        let shown = import_log[import_log.len().saturating_sub(25)..].join("\n");
        let build = self.build.clone();
        self.shutdown().await;
        error.context(format!(
            "Kotlin language server (build {build}) was not ready.\nLast Gradle import output:\n{shown}"
        ))
    }

    pub fn pid(&self) -> Option<u32> {
        self.session.pid()
    }

    /// A request that fails loudly instead of waiting forever. An error
    /// answer stays a downcastable `RpcError`.
    pub async fn request(&mut self, method: &str, params: Value) -> Result<Value> {
        tokio::time::timeout(self.timeout, self.session.request(method, params))
            .await
            .map_err(|_| {
                anyhow::anyhow!(
                    "The Kotlin language server did not answer {method} within {} seconds",
                    self.timeout.as_secs()
                )
            })?
    }

    pub async fn notify(&mut self, method: &str, params: Value) -> Result<()> {
        self.session.notify(method, params).await
    }

    pub async fn sync_document(&mut self, path: &Path, text: &str) -> Result<()> {
        self.session.sync_document(path, text).await
    }

    pub async fn shutdown(self) {
        self.session.shutdown().await;
        // Best effort: the server may still hold a file for a moment.
        let _ = self.system_dir.close();
    }
}

fn client_capabilities() -> Value {
    json!({
        "workspace": {
            "applyEdit": true,
            "workspaceEdit": {
                "documentChanges": true,
                "resourceOperations": ["create", "rename", "delete"],
            },
            "workspaceFolders": true,
            "configuration": true,
            "fileOperations": { "willRename": true },
        },
        "window": { "workDoneProgress": true },
        "textDocument": {
            "rename": { "prepareSupport": true },
            "documentSymbol": { "hierarchicalDocumentSymbolSupport": true },
        },
    })
}

fn check_capabilities(capabilities: &Value) -> Result<()> {
    if capabilities["workspace"]["fileOperations"]["willRename"].is_null() {
        bail!("The Kotlin language server does not offer workspace/willRenameFiles");
    }
    if capabilities["renameProvider"]["prepareProvider"] != true {
        bail!("The Kotlin language server does not offer prepareRename");
    }
    if capabilities["referencesProvider"].is_null()
        || capabilities["documentSymbolProvider"].is_null()
    {
        bail!("The Kotlin language server does not offer references and document symbols");
    }
    Ok(())
}

fn language_id(path: &Path) -> &'static str {
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("java") => "java",
        _ => "kotlin",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn install_dir(build_txt: Option<&str>, with_executable: bool) -> TempDir {
        let dir = tempfile::tempdir().unwrap();
        if let Some(text) = build_txt {
            std::fs::write(dir.path().join("build.txt"), text).unwrap();
        }
        if with_executable {
            std::fs::create_dir(dir.path().join("bin")).unwrap();
            std::fs::write(dir.path().join("bin/intellij-server"), "").unwrap();
        }
        dir
    }

    #[test]
    fn a_missing_setting_explains_the_install() {
        let error = locate_in(None).err().unwrap().to_string();
        assert!(error.contains(SERVER_ENV), "{error}");
        assert!(error.contains(DOWNLOAD_URL), "{error}");
        assert!(error.contains(DOWNLOAD_SHA256), "{error}");
    }

    #[test]
    fn a_folder_without_build_txt_is_refused() {
        let dir = install_dir(None, true);
        let error = locate_in(Some(dir.path().into()))
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("no build.txt"), "{error}");
    }

    #[test]
    fn a_foreign_build_is_refused() {
        let dir = install_dir(Some("IC-263.1"), true);
        let error = locate_in(Some(dir.path().into()))
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("expected ILS-"), "{error}");
    }

    #[test]
    fn a_missing_executable_is_refused() {
        let dir = install_dir(Some("ILS-263.6379.0\n"), false);
        let error = locate_in(Some(dir.path().into()))
            .err()
            .unwrap()
            .to_string();
        assert!(error.contains("intellij-server"), "{error}");
    }

    #[test]
    fn a_complete_install_reports_its_build() {
        let dir = install_dir(Some("ILS-263.6379.0\n"), true);
        let install = locate_in(Some(dir.path().into())).unwrap();
        assert_eq!(install.build, "263.6379.0");
    }

    #[test]
    fn the_timeout_defaults_and_rejects_nonsense() {
        assert_eq!(parse_timeout(None).unwrap(), Duration::from_secs(600));
        assert_eq!(parse_timeout(Some("90")).unwrap(), Duration::from_secs(90));
        assert!(parse_timeout(Some("0")).is_err());
        assert!(parse_timeout(Some("soon")).is_err());
    }
}
