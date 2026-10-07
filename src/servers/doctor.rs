//! `refac doctor`: for each language server, whether refac can use it, what
//! it looked at, and what to do when it cannot. The commands that start a
//! server report a missing one with a pointer to this command, so an agent
//! that hits the problem can fix it without asking the user.

mod render;

use super::handshake::{StartCheck, start_check};
use super::locate::{Attempt, Located, locate};
use super::{Requirement, Server, all, for_language};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub use render::{render_json, render_one, render_overview};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Ready,
    Missing,
    /// Found, but it does not run.
    Broken,
}

pub struct Report {
    pub server: &'static Server,
    pub status: Status,
    pub located: Option<Located>,
    /// Every place looked at, in order.
    pub attempts: Vec<Attempt>,
    /// Each needed tool and where it is, if it is there.
    pub requirements: Vec<(&'static Requirement, Option<PathBuf>)>,
    /// Only made by `doctor_one`: starting the server proves it works.
    pub start: Option<StartCheck>,
}

fn report(server: &'static Server, project: &Path) -> Report {
    let requirements = server
        .requirements
        .iter()
        .map(|requirement| (requirement, which::which(requirement.executable).ok()))
        .collect();
    match locate(server, project) {
        Ok(located) => Report {
            server,
            status: Status::Ready,
            attempts: located.attempts.clone(),
            located: Some(located),
            requirements,
            start: None,
        },
        Err(not_found) => {
            let broken = not_found
                .attempts
                .iter()
                .any(|attempt| matches!(attempt.outcome, super::Outcome::Broken(_)));
            Report {
                server,
                status: if broken {
                    Status::Broken
                } else {
                    Status::Missing
                },
                located: None,
                attempts: not_found.attempts,
                requirements,
                start: None,
            }
        }
    }
}

/// One line per language: no server is started.
pub fn doctor_all(project: &Path) -> Vec<Report> {
    all().iter().map(|server| report(server, project)).collect()
}

/// The details for one language, including a start-and-stop of the server.
pub async fn doctor_one(language: &str, project: &Path) -> Result<Report> {
    let server = for_language(language)?;
    let mut report = report(server, project);
    if let Some(located) = &report.located {
        let check = start_check(server, located, project).await;
        if matches!(check, StartCheck::Failed(_)) {
            report.status = Status::Broken;
        }
        report.start = Some(check);
    }
    Ok(report)
}
