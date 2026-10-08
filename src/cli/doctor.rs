//! `refac doctor`: whether each language server refac needs is installed and
//! works, and how to install it when it is not. Meant for an agent that hit
//! "server not found" and wants to fix it alone.

use super::{CliError, write_json};
use crate::servers::{self, Status};
use clap::{Args, ValueHint};
use std::io;

#[derive(Debug, Args)]
pub struct DoctorArgs {
    /// The language to check: go, rust, python, dart, kotlin or typescript.
    ///
    /// Without it, one line per language is printed and no server is started. With it, the server
    /// is found, started, asked to answer and stopped, and the install steps are printed if it is
    /// missing or broken.
    #[arg(value_name = "LANGUAGE")]
    pub language: Option<String>,

    /// The project the server will work in (default: the current directory).
    ///
    /// rustup picks the rust-analyzer of the project's toolchain from there, and Python its
    /// environment.
    #[arg(long, value_name = "DIR", value_hint = ValueHint::DirPath, env = "REFAC_PROJECT_PATH")]
    pub project_path: Option<std::path::PathBuf>,

    /// Print a JSON list (one object per language) instead of text.
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
