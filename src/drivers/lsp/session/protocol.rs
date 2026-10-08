//! The wire format of a stdio language server: `Content-Length` framed JSON,
//! and the typed error answer.

use anyhow::{Context, Result, bail};
use serde_json::Value;
use std::fmt;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, BufReader};
use tokio::process::ChildStdout;

/// A JSON-RPC error answer, kept typed so a refused request (an expected
/// outcome) is distinguishable from a broken server.
#[derive(Debug)]
pub struct RpcError {
    pub code: i64,
    pub message: String,
}

impl fmt::Display for RpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (LSP error {})", self.message, self.code)
    }
}

impl std::error::Error for RpcError {}

pub async fn read_message(reader: &mut BufReader<ChildStdout>) -> Result<Value> {
    let mut length = None;
    loop {
        let mut line = String::new();
        if reader.read_line(&mut line).await? == 0 {
            bail!("Server closed its output");
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some((name, value)) = line.split_once(':')
            && name.eq_ignore_ascii_case("content-length")
        {
            length = Some(value.trim().parse::<usize>()?);
        }
    }
    let mut body = vec![0; length.context("Message without Content-Length")?];
    reader.read_exact(&mut body).await?;
    Ok(serde_json::from_slice(&body)?)
}
