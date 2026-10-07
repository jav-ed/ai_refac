use anyhow::{Context, Result, bail};
use std::process::{Output, Stdio};
use std::time::Duration;
use sysinfo::{Pid, ProcessRefreshKind, ProcessesToUpdate, System};
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::Command;

const MIB: u64 = 1024 * 1024;
const MEMORY_ENV: &str = "REFAC_TYPESCRIPT_MAX_RSS_MB";

pub(super) struct Limits {
    pub timeout: Duration,
    pub rss_bytes: u64,
}

impl Limits {
    pub fn from_env() -> Result<Self> {
        let value = match std::env::var(MEMORY_ENV) {
            Ok(value) => value,
            Err(std::env::VarError::NotPresent) => "4096".to_owned(),
            Err(error) => return Err(error).context(MEMORY_ENV),
        };
        let mib = value.parse::<u64>().ok().filter(|value| *value > 0);
        let rss_bytes = mib
            .and_then(|value| value.checked_mul(MIB))
            .with_context(|| {
                format!("{MEMORY_ENV} must be a positive integer in MiB; received {value:?}")
            })?;
        Ok(Self {
            timeout: Duration::from_secs(300),
            rss_bytes,
        })
    }
}

async fn read(mut pipe: impl AsyncRead + Unpin) -> std::io::Result<Vec<u8>> {
    let mut bytes = Vec::new();
    pipe.read_to_end(&mut bytes).await?;
    Ok(bytes)
}

pub(super) async fn memory_limit(pid: Pid, limit: u64) -> Result<()> {
    let mut system = System::new();
    let mut interval = tokio::time::interval(Duration::from_millis(100));
    loop {
        interval.tick().await;
        system.refresh_processes_specifics(
            ProcessesToUpdate::Some(&[pid]),
            true,
            ProcessRefreshKind::nothing().with_memory(),
        );
        // A disappearing process is handled by wait(), including its exit status.
        let Some(process) = system.process(pid) else {
            continue;
        };
        if process.memory() > limit {
            bail!(
                "TypeScript helper exceeded its RAM limit: {} MiB RSS > {} MiB ({MEMORY_ENV})",
                process.memory() / MIB,
                limit / MIB
            );
        }
    }
}

pub(super) async fn run(command: &mut Command, limits: Limits) -> Result<Output> {
    anyhow::ensure!(
        sysinfo::IS_SUPPORTED_SYSTEM,
        "TypeScript helper RAM monitoring is unsupported on this operating system"
    );
    // Dropping a timed-out output() future does not stop its process. Keep the
    // child handle so failures explicitly kill AND reap it; cancellation kills too.
    let mut child = command
        .kill_on_drop(true)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Could not start TypeScript helper")?;
    let pid = Pid::from_u32(child.id().context("TypeScript helper has no PID")?);
    let stdout = child
        .stdout
        .take()
        .context("TypeScript helper stdout was not piped")?;
    let stderr = child
        .stderr
        .take()
        .context("TypeScript helper stderr was not piped")?;
    let outcome = tokio::select! {
        result = async { tokio::try_join!(child.wait(), read(stdout), read(stderr)) } => {
            result.context("Could not collect TypeScript helper result")
        }
        _ = tokio::time::sleep(limits.timeout) => {
            Err(anyhow::anyhow!("TypeScript helper timed out after {} seconds", limits.timeout.as_secs_f64()))
        }
        result = memory_limit(pid, limits.rss_bytes) => {
            result.and_then(|_| Err(anyhow::anyhow!("TypeScript RAM monitor stopped unexpectedly")))
        }
    };
    match outcome {
        Ok((status, stdout, stderr)) => Ok(Output {
            status,
            stdout,
            stderr,
        }),
        Err(error) => {
            child
                .kill()
                .await
                .context("Failed to terminate and reap TypeScript helper")?;
            bail!(
                "{error:#}. TypeScript helper terminated and reaped. The move did not complete; inspect the working tree before retrying"
            )
        }
    }
}

#[cfg(test)]
mod tests;
