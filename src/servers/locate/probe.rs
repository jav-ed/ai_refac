//! Proving a candidate runs: its version command, with a time limit. A place
//! counts as a server only when this succeeds, because a stand-in that exists
//! but fails (rustup's `rust-analyzer` without the component) is worse than
//! none.

use crate::servers::{Candidate, Version};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

/// A command that proves a server runs must finish quickly.
const PROBE_TIMEOUT: Duration = Duration::from_secs(20);

/// Run the version command of a candidate. `Ok` carries the first line it
/// printed; `Err` the reason it does not work.
pub(super) fn probe(
    executable: &Path,
    candidate: &Candidate,
    project: &Path,
) -> Result<String, String> {
    let (program, args): (PathBuf, &[&str]) = match &candidate.version {
        Version::Own(args) => (executable.to_path_buf(), args),
        Version::Companion(name, args) => (
            executable
                .parent()
                .map(|dir| dir.join(name))
                .filter(|path| path.is_file())
                .unwrap_or_else(|| PathBuf::from(name)),
            args,
        ),
    };
    let command = format!("{} {}", program.display(), args.join(" "));
    let output = run_with_timeout(
        Command::new(&program).args(args).current_dir(project),
        PROBE_TIMEOUT,
    )
    .map_err(|error| format!("`{}` cannot run: {error}", command.trim()))?;
    let text = |bytes: &[u8]| String::from_utf8_lossy(bytes).trim().to_string();
    if !output.status.success() {
        let said = text(&output.stderr);
        let said = if said.is_empty() {
            text(&output.stdout)
        } else {
            said
        };
        let first = said.lines().next().unwrap_or("it printed nothing");
        return Err(format!("`{}` failed: {first}", command.trim()));
    }
    let said = text(&output.stdout);
    let said = if said.is_empty() {
        text(&output.stderr)
    } else {
        said
    };
    Ok(said.lines().next().unwrap_or("").to_string())
}

pub(super) fn run_with_timeout(
    command: &mut Command,
    timeout: Duration,
) -> std::io::Result<std::process::Output> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let deadline = Instant::now() + timeout;
    loop {
        if child.try_wait()?.is_some() {
            return child.wait_with_output();
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            let _ = child.wait();
            return Err(std::io::Error::other(format!(
                "it did not finish within {} seconds",
                timeout.as_secs_f32().ceil()
            )));
        }
        std::thread::sleep(Duration::from_millis(20));
    }
}
