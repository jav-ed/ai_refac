//! What a language server that went away leaves behind to explain itself: how
//! its process ended and the last lines it wrote to stderr. Without them a
//! crash reads "closed its connection" and nobody can tell a memory kill from
//! a broken installation, which is the one thing a failed command has to say.

use super::LspSession;
use anyhow::{Context, Result};
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;
use std::process::{ExitStatus, Stdio};
use std::time::Duration;
use tempfile::NamedTempFile;
use tokio::process::Child;

/// How much of the end of the stderr file is read back.
const TAIL_BYTES: u64 = 8 * 1024;
/// How many of its last lines are shown, each cut to `LINE_CHARS`.
const TAIL_LINES: usize = 20;
const LINE_CHARS: usize = 300;
/// A server that closed its connection is given this long to finish ending,
/// so the exit status is there to read.
const ENDING: Duration = Duration::from_secs(3);

/// The file a server's stderr goes to (a file, not a pipe: nothing has to
/// read it while the server runs, and a crash can be explained afterwards) and
/// the handle the process is started with. The file is deleted with the session.
pub(super) fn stderr_file(name: &str) -> Result<(NamedTempFile, Stdio)> {
    let file = tempfile::Builder::new()
        .prefix("refac-server-stderr-")
        .tempfile()
        .with_context(|| format!("Cannot create a file for the {name} stderr"))?;
    let handle = std::fs::OpenOptions::new()
        .append(true)
        .open(file.path())
        .with_context(|| format!("Cannot open the {name} stderr file"))?;
    Ok((file, Stdio::from(handle)))
}

impl LspSession {
    /// The error for a server that went away: what we were doing, how its
    /// process ended and what it last wrote on stderr.
    pub(super) async fn gone(&mut self, what: &str) -> anyhow::Error {
        let detail = describe(&mut self.child, self.stderr_log.path()).await;
        anyhow::anyhow!("The {} {what}. {detail}", self.name)
    }
}

/// How the process ended and what it last wrote on stderr (`stderr` is the
/// file the server was started with).
async fn describe(child: &mut Child, stderr: &Path) -> String {
    let ended = match tokio::time::timeout(ENDING, child.wait()).await {
        Ok(Ok(status)) => status_text(status),
        Ok(Err(error)) => format!("could not be waited for ({error})"),
        Err(_) => format!(
            "was still running {} s after it closed its connection",
            ENDING.as_secs()
        ),
    };
    match tail(stderr) {
        Ok(lines) if lines.is_empty() => {
            format!("The process {ended}. It wrote nothing to stderr.")
        }
        Ok(lines) => format!("The process {ended}. Its last lines on stderr:\n{lines}"),
        Err(error) => format!("The process {ended}. Its stderr could not be read back ({error})."),
    }
}

fn status_text(status: ExitStatus) -> String {
    #[cfg(unix)]
    if let Some(signal) = std::os::unix::process::ExitStatusExt::signal(&status) {
        return format!("was killed by {}", signal_text(signal));
    }
    match status.code() {
        Some(code) => format!("exited with status {code}"),
        None => "ended without an exit status".to_string(),
    }
}

fn signal_text(signal: i32) -> String {
    match signal {
        6 => "signal 6 (SIGABRT: it aborted itself, as a JVM does on a fatal error)".to_string(),
        9 => "signal 9 (SIGKILL: on Linux this is usually the out-of-memory killer; `dmesg` or `journalctl -k` names it, and the machine needs more free memory or swap)".to_string(),
        11 => "signal 11 (SIGSEGV: it crashed)".to_string(),
        15 => "signal 15 (SIGTERM: something asked it to stop)".to_string(),
        other => format!("signal {other}"),
    }
}

/// The last lines of the file, indented, oldest first.
fn tail(path: &Path) -> std::io::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let start = file.metadata()?.len().saturating_sub(TAIL_BYTES);
    file.seek(SeekFrom::Start(start))?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;
    let text = String::from_utf8_lossy(&bytes);
    let mut lines: Vec<&str> = text.lines().collect();
    // Reading started in the middle of a line.
    if start > 0 && !lines.is_empty() {
        lines.remove(0);
    }
    let lines: Vec<String> = lines
        .into_iter()
        .map(str::trim_end)
        .filter(|line| !line.is_empty())
        .map(|line| format!("  {}", line.chars().take(LINE_CHARS).collect::<String>()))
        .collect();
    Ok(lines[lines.len().saturating_sub(TAIL_LINES)..].join("\n"))
}

#[cfg(test)]
mod tests;
