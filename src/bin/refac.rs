use refac::cli;
use tracing_subscriber::{self, EnvFilter};

#[tokio::main]
async fn main() -> std::process::ExitCode {
    // rust-analyzer's libraries are intentionally verbose at info level. Keep
    // normal CLI output focused while still honoring an explicit RUST_LOG.
    let filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("warn,refac=info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .with_ansi(false)
        .init();

    cli::run().await
}
