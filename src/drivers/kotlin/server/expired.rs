//! The error for a server build whose early-access license ran out.
//!
//! JetBrains publishes the Kotlin server only as early-access builds, and each
//! one stops starting at a fixed date, about four weeks after its release: it
//! prints "This build of kotlin-server has expired" and exits with status 7,
//! the exit code the server uses for licensing problems. The bare start error
//! only says that the connection closed, so nobody would learn that the cure is
//! a newer download, or where to read how to get one.

use crate::servers::KOTLIN_SETUP_GUIDE;
use anyhow::anyhow;
use std::path::Path;

/// What the server prints when its date has passed.
const EXPIRED_TEXT: &str = "This build of kotlin-server has expired";
/// How `LspSession` words the exit status of a server that left with the
/// licensing exit code.
const LICENSING_EXIT: &str = "exited with status 7";

/// `error` is the failure of the server's start. A licensing failure becomes
/// the message below, with the server's own report at the end; any other
/// failure is returned as it is.
pub(super) fn explain(error: anyhow::Error, executable: &Path) -> anyhow::Error {
    let report = format!("{error:#}");
    let expired = report.contains(EXPIRED_TEXT);
    if !expired && !report.contains(LICENSING_EXIT) {
        return error;
    }
    let reason = if expired {
        "this build has expired"
    } else {
        "its license was refused (exit status 7: an expired build, or a feature the license does not cover)"
    };
    anyhow!(
        "The Kotlin language server did not start: {reason}.\n\
         \x20 Server: {server}\n\
         JetBrains publishes this server only as early-access builds. Each one stops starting at a fixed date, about four weeks after its release. This is not a problem of your project or of refac: the installed build is out of date.\n\
         \n\
         What to do:\n\
         \x20 1. Run `refac doctor kotlin`. It prints the build refac was verified with, its download, its checksum and the install steps.\n\
         \x20 2. Install that build and point REFAC_KOTLIN_SERVER at its folder.\n\
         \x20 3. If `refac doctor kotlin` shows that build as expired too, the refac repository needs a newer one. How to get and check a newer build is in the refac repository, in Project_Manag/Docs/Setup/kotlin_Server.md (\"Replacing the build\"):\n\
         \x20    {KOTLIN_SETUP_GUIDE}\n\
         \x20    New builds are published at https://github.com/Kotlin/kotlin-lsp/releases\n\
         \n\
         What the server printed:\n\
         {report}",
        server = executable.display()
    )
}

#[cfg(test)]
mod tests;
