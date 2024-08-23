use std::{process::Stdio, string::FromUtf8Error};
use thiserror::Error;
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    process::Command,
};

#[derive(Error, Debug)]
pub enum ProcessError {
    #[error("failed to spawn process: {0}")]
    Spawn(#[from] io::Error),

    #[error("failed to open stdin")]
    Stdin,

    #[error("failed to open stdout")]
    Stdout,

    #[error("failed to parse stderr")]
    StderrParse,

    #[error("podman error: {0}")]
    Podman(String),

    #[error("UTF-8 parse error: {0}")]
    FromUtf8(#[from] FromUtf8Error),
}

pub async fn run(mut command: Command, input: Option<String>) -> Result<String, ProcessError> {
    command
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::piped());

    let mut child = command.spawn()?;

    if let Some(input) = input {
        let mut stdin = child.stdin.take().ok_or(ProcessError::Stdin)?;
        stdin.write_all(input.as_bytes()).await?;
        stdin.flush().await?;
    }

    let mut stdout = child.stdout.take().ok_or(ProcessError::Stdout)?;
    let mut output_bytes = Vec::new();
    stdout.read_to_end(&mut output_bytes).await?;
    let output = String::from_utf8(output_bytes)?;

    let status = child.wait().await?;
    if !status.success() {
        let mut stderr = child.stderr.take().ok_or(ProcessError::StderrParse)?;
        let mut error_message = Vec::new();
        stderr.read_to_end(&mut error_message).await?;
        let error_message = String::from_utf8(error_message)?;
        Err(ProcessError::Podman(error_message))
    } else {
        Ok(output)
    }
}
