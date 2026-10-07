//! The two shapes of the doctor's answer: text for a person or an agent
//! reading the terminal, and JSON for a script.

use super::{Report, Status};
use crate::servers::handshake::StartCheck;
use crate::servers::locate::describe;
use serde_json::{Value, json};

fn label(status: Status) -> &'static str {
    match status {
        Status::Ready => "ready",
        Status::Missing => "MISSING",
        Status::Broken => "BROKEN",
    }
}

fn first_line(text: &str) -> &str {
    text.lines().next().unwrap_or("")
}

pub fn render_overview(reports: &[Report]) -> String {
    let mut out = String::from(
        "Language servers refac starts on demand. It stops each one when its command ends, so none stays in memory.\n\n",
    );
    for report in reports {
        let detail = match &report.located {
            Some(located) => format!(
                "{} ({})",
                first_line(&located.version),
                located.executable.display()
            ),
            None => format!("{}: not usable", report.server.name),
        };
        out.push_str(&format!(
            "  {:<11} {:<8} {}\n",
            report.server.languages[0],
            label(report.status),
            detail
        ));
    }
    out.push_str("\nSteps for one language, and a start check: refac doctor <language>\n");
    out
}

pub fn render_one(report: &Report) -> String {
    let server = report.server;
    let mut out = format!(
        "{}: {} [{}]\n  Used for: {}.\n  refac starts it for each command and stops it afterwards; nothing stays running.\n",
        server.language,
        server.name,
        label(report.status),
        server.used_for
    );
    if let Some(located) = &report.located {
        out.push_str(&format!(
            "  Found: {} (via {}), {}\n",
            located.executable.display(),
            located.via,
            first_line(&located.version)
        ));
    }
    match &report.start {
        Some(StartCheck::Started { seconds }) => out.push_str(&format!(
            "  Start check: it started, answered, and stopped in {seconds:.1} s.\n"
        )),
        Some(StartCheck::Failed(reason)) => {
            out.push_str(&format!("  Start check FAILED: {reason}\n"))
        }
        Some(StartCheck::Skipped(reason)) => {
            out.push_str(&format!("  Start check skipped: {reason}.\n"))
        }
        None => {}
    }
    for (requirement, found) in &report.requirements {
        match found {
            Some(path) => out.push_str(&format!(
                "  Needs {}: found {}\n",
                requirement.name,
                path.display()
            )),
            None => out.push_str(&format!(
                "  Needs {}: NOT FOUND ({}); {}\n",
                requirement.name, requirement.why, requirement.hint
            )),
        }
    }
    if report.status != Status::Ready {
        out.push_str("  Looked here:\n");
        for attempt in &report.attempts {
            out.push_str(&format!("    - {}\n", describe(attempt)));
        }
    }
    if report.status != Status::Ready || report.start.is_none() {
        out.push_str("  Install:\n");
        for (index, step) in server.install.iter().enumerate() {
            out.push_str(&format!("    {}. {}\n", index + 1, step));
        }
        if !server.env_var.is_empty() {
            out.push_str(&format!(
                "  The variable {} beats every other place when it is set.\n",
                server.env_var
            ));
        }
        out.push_str(&format!(
            "  Check again with: refac doctor {}\n",
            server.languages[0]
        ));
    }
    out
}

pub fn render_json(reports: &[Report]) -> Value {
    json!(reports.iter().map(json_of).collect::<Vec<_>>())
}

fn json_of(report: &Report) -> Value {
    let server = report.server;
    json!({
        "language": server.languages[0],
        "server": server.name,
        "status": label(report.status).to_lowercase(),
        "used_for": server.used_for,
        "env_var": server.env_var,
        "executable": report.located.as_ref().map(|found| found.executable.display().to_string()),
        "version": report.located.as_ref().map(|found| first_line(&found.version).to_string()),
        "looked": report.attempts.iter().map(describe).collect::<Vec<_>>(),
        "requirements": report.requirements.iter().map(|(requirement, found)| json!({
            "name": requirement.name,
            "found": found.as_ref().map(|path| path.display().to_string()),
            "hint": requirement.hint,
        })).collect::<Vec<_>>(),
        "start_check": report.start.as_ref().map(|check| match check {
            StartCheck::Started { seconds } => json!({ "result": "ok", "seconds": seconds }),
            StartCheck::Failed(reason) => json!({ "result": "failed", "reason": reason }),
            StartCheck::Skipped(reason) => json!({ "result": "skipped", "reason": reason }),
        }),
        "install": server.install,
    })
}
