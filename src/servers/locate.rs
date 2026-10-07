//! Searching for a server in a fixed order and keeping a record of every
//! place looked at, so a failure can say what was tried and not found.
//!
//! Order: the environment variable, then PATH, then the folders installers
//! use. A place counts only when the server there runs, because a stand-in
//! that exists but fails (rustup's `rust-analyzer` without the component) is
//! worse than none. A variable that is set but wrong is an error on its own;
//! falling back would hide a mistake.

use super::{Candidate, Find, Server};
use probe::probe;
mod probe;

use std::ffi::OsString;
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub enum Outcome {
    Found(PathBuf),
    /// Nothing is there; the text says what was looked for.
    NotThere(String),
    /// Something is there but does not work; the text says why.
    Broken(String),
}

#[derive(Debug, Clone)]
pub struct Attempt {
    pub place: String,
    pub outcome: Outcome,
}

/// A server that was found and runs.
#[derive(Debug, Clone)]
pub struct Located {
    pub executable: PathBuf,
    /// What the server printed when asked for its version, first line.
    pub version: String,
    /// Where it was found: "$REFAC_GOPLS", "PATH", or a folder.
    pub via: String,
    /// For a downloaded folder (Kotlin), the folder.
    pub folder: Option<PathBuf>,
    pub attempts: Vec<Attempt>,
}

pub struct NotFound {
    pub server: &'static Server,
    pub attempts: Vec<Attempt>,
}

impl std::error::Error for NotFound {}

impl fmt::Debug for NotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "NotFound({})", self.server.name)
    }
}

impl NotFound {
    fn broken(&self) -> bool {
        self.attempts
            .iter()
            .any(|attempt| matches!(attempt.outcome, Outcome::Broken(_)))
    }

    /// The command that teaches how to fix it.
    pub fn help_command(&self) -> String {
        format!("refac doctor {}", self.server.languages[0])
    }
}

impl fmt::Display for NotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let server = self.server;
        if self.broken() {
            writeln!(
                f,
                "The {} language server ({}) is installed but does not work.",
                server.language, server.name
            )?;
        } else {
            writeln!(
                f,
                "The {} language server ({}) was not found.",
                server.language, server.name
            )?;
        }
        writeln!(
            f,
            "refac starts it by itself for each command and stops it afterwards, so no server has to be running; it has to be installed. Looked here:"
        )?;
        for attempt in &self.attempts {
            writeln!(f, "  - {}", describe(attempt))?;
        }
        write!(
            f,
            "Run `{}` to see how to install or fix it, then repeat the command.",
            self.help_command()
        )
    }
}

/// "place: what happened", the line that doctor and the error both print.
pub fn describe(attempt: &Attempt) -> String {
    match &attempt.outcome {
        Outcome::Found(path) => format!("{}: found {}", attempt.place, path.display()),
        Outcome::NotThere(reason) => format!("{}: {reason}", attempt.place),
        Outcome::Broken(reason) => format!("{}: {reason}", attempt.place),
    }
}

/// The working server, or the record of where it is not. `project` is the
/// directory refac works in: toolchain pins and project environments are
/// found from there.
pub fn locate(server: &'static Server, project: &Path) -> Result<Located, NotFound> {
    locate_with(server, project, std::env::var_os(server.env_var))
}

/// `locate` with the value of the server's environment variable given, so the
/// search can be tried without touching the process environment.
pub fn locate_with(
    server: &'static Server,
    project: &Path,
    configured: Option<OsString>,
) -> Result<Located, NotFound> {
    let mut attempts = Vec::new();
    let found = match &server.find {
        Find::Executable {
            candidates,
            folders,
            ..
        } => search_executables(
            server,
            candidates,
            &folders(project),
            project,
            configured,
            &mut attempts,
        ),
        Find::Folder {
            executable,
            build_file,
        } => search_folder(server, executable, build_file, configured, &mut attempts),
        Find::Bundled { check } => {
            attempts = check();
            attempts.iter().find_map(|attempt| match &attempt.outcome {
                Outcome::Found(path) => Some(Located {
                    executable: path.clone(),
                    version: "installed".to_string(),
                    via: attempt.place.clone(),
                    folder: None,
                    attempts: Vec::new(),
                }),
                _ => None,
            })
        }
    };
    match found {
        Some(mut located) => {
            located.attempts = attempts;
            Ok(located)
        }
        None => Err(NotFound { server, attempts }),
    }
}

fn search_executables(
    server: &Server,
    candidates: &[Candidate],
    folders: &[PathBuf],
    project: &Path,
    configured: Option<OsString>,
    attempts: &mut Vec<Attempt>,
) -> Option<Located> {
    let variable = format!("${}", server.env_var);
    if let Some(value) = configured {
        let path = PathBuf::from(&value);
        // The variable names a file, whichever candidate it is.
        let outcome = candidates
            .iter()
            .map(|candidate| probe(&path, candidate, project))
            .find(|outcome| outcome.is_ok())
            .unwrap_or_else(|| probe(&path, &candidates[0], project));
        return match outcome {
            Ok(version) => {
                attempts.push(found(&variable, &path));
                Some(located(path, version, variable))
            }
            Err(reason) => {
                // A wrong override is the user's mistake; do not look elsewhere.
                attempts.push(Attempt {
                    place: format!("{variable} is set to {}", path.display()),
                    outcome: Outcome::Broken(reason),
                });
                None
            }
        };
    }
    attempts.push(Attempt {
        place: variable,
        outcome: Outcome::NotThere("not set".to_string()),
    });

    for candidate in candidates {
        match which::which(candidate.executable) {
            Ok(path) => match probe(&path, candidate, project) {
                Ok(version) => {
                    attempts.push(found("PATH", &path));
                    return Some(located(path, version, "PATH".to_string()));
                }
                Err(reason) => attempts.push(Attempt {
                    place: format!("PATH ({})", path.display()),
                    outcome: Outcome::Broken(reason),
                }),
            },
            Err(_) => attempts.push(Attempt {
                place: "PATH".to_string(),
                outcome: Outcome::NotThere(format!("no command named `{}`", candidate.executable)),
            }),
        }
    }
    for folder in folders {
        for candidate in candidates {
            let path = folder.join(candidate.executable);
            let place = path.display().to_string();
            if !path.is_file() {
                attempts.push(Attempt {
                    place,
                    outcome: Outcome::NotThere("no such file".to_string()),
                });
                continue;
            }
            match probe(&path, candidate, project) {
                Ok(version) => {
                    attempts.push(found(&place, &path));
                    return Some(located(path, version, folder.display().to_string()));
                }
                Err(reason) => attempts.push(Attempt {
                    place,
                    outcome: Outcome::Broken(reason),
                }),
            }
        }
    }
    None
}

fn found(place: &str, path: &Path) -> Attempt {
    Attempt {
        place: place.to_string(),
        outcome: Outcome::Found(path.to_path_buf()),
    }
}

fn located(executable: PathBuf, version: String, via: String) -> Located {
    Located {
        executable,
        version,
        via,
        folder: None,
        attempts: Vec::new(),
    }
}

/// A downloaded folder: the variable names it, and it must hold the
/// executable and the file that names the build.
fn search_folder(
    server: &Server,
    executable: &str,
    build_file: &str,
    configured: Option<OsString>,
    attempts: &mut Vec<Attempt>,
) -> Option<Located> {
    let variable = format!("${}", server.env_var);
    let Some(value) = configured else {
        attempts.push(Attempt {
            place: variable,
            outcome: Outcome::NotThere("not set".to_string()),
        });
        return None;
    };
    let folder = PathBuf::from(value);
    let place = format!("{variable} is set to {}", folder.display());
    let build = match std::fs::read_to_string(folder.join(build_file)) {
        Ok(build) => build.trim().to_string(),
        Err(_) => {
            attempts.push(Attempt {
                place,
                outcome: Outcome::Broken(format!(
                    "there is no {build_file} in it, so it is not an unpacked server download"
                )),
            });
            return None;
        }
    };
    let program = folder.join(executable);
    if !program.is_file() {
        attempts.push(Attempt {
            place,
            outcome: Outcome::Broken(format!("{executable} is missing from it")),
        });
        return None;
    }
    attempts.push(found(&variable, &folder));
    Some(Located {
        executable: program,
        version: build,
        via: variable,
        folder: Some(folder),
        attempts: Vec::new(),
    })
}

#[cfg(test)]
mod tests;
