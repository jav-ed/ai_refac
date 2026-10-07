//! `refac doctor`: whether each language server refac needs is installed and
//! works, and how to install it when it is not. Meant for an agent that hit
//! "server not found" and wants to fix it alone.

use super::{CliError, write_json};
use crate::servers::{self, Status};
use clap::{Args, ValueHint};
use std::io;

#[derive(Debug, Args)]
pub struct DoctorArgs {
    /// Language to check and explain: go, rust, python, dart, kotlin, typescript. Without it, one line per language and no server is started.
    pub language: Option<String>,

    /// Project the server will work in (rustup picks its toolchain and Python its environment from there). Defaults to the current directory.
    #[arg(long, value_hint = ValueHint::DirPath, env = "REFAC_PROJECT_PATH")]
    pub project_path: Option<std::path::PathBuf>,

    /// Emit machine-readable JSON instead of human text.
    #[arg(long)]
    pub json: bool,
}

pub async fn execute_doctor(args: DoctorArgs) -> Result<(), CliError> {
    let json = args.json;
    let fail = |error: anyhow::Error| CliError { json, error };
    let project = args
        .project_path
        .map(Ok)
        .unwrap_or_else(std::env::current_dir)
        .map_err(|error| fail(error.into()))?;

    let Some(language) = args.language else {
        let reports = servers::doctor_all(&project);
        if json {
            write_json(io::stdout(), &servers::render_json(&reports)).map_err(fail)?;
        } else {
            print!("{}", servers::render_overview(&reports));
        }
        return Ok(());
    };

    let report = servers::doctor_one(&language, &project)
        .await
        .map_err(fail)?;
    if json {
        write_json(
            io::stdout(),
            &servers::render_json(std::slice::from_ref(&report)),
        )
        .map_err(fail)?;
    } else {
        print!("{}", servers::render_one(&report));
    }
    // A script can branch on the exit code; the details were printed above.
    match report.status {
        Status::Ready => Ok(()),
        Status::Missing | Status::Broken => Err(fail(anyhow::anyhow!(
            "{} ({}) is not ready; the steps are above",
            report.server.language,
            report.server.name
        ))),
    }
}
