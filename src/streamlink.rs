use std::process::{Command as StdCommand, Stdio};

use anyhow::{bail, Context, Result};
use tokio::process::Command;

const STREAMLINK_ARGS: [&str; 4] = ["--twitch-disable-ads", "--player", "mpv", "-a"];
const STREAMLINK_PLAYER_ARGS: &str = "--cache=yes --cache-secs=600";

pub fn ensure_dependencies() -> Result<()> {
    ensure_command_available("streamlink")?;
    ensure_command_available("mpv")?;
    Ok(())
}

pub async fn launch(url: &str) -> Result<()> {
    let status = Command::new("streamlink")
        .args(STREAMLINK_ARGS)
        .arg(STREAMLINK_PLAYER_ARGS)
        .arg(url)
        .arg("best")
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .await
        .context("failed to start streamlink")?;

    if !status.success() {
        bail!("streamlink exited with status {}", status);
    }

    Ok(())
}

pub fn spawn(url: &str) -> Result<tokio::process::Child> {
    let mut cmd = Command::new("streamlink");
    cmd.args(STREAMLINK_ARGS)
        .arg(STREAMLINK_PLAYER_ARGS)
        .arg(url)
        .arg("best")
        .stdin(Stdio::null())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit());

    cmd.spawn()
        .with_context(|| format!("failed to start streamlink for {}", url))
}

fn ensure_command_available(name: &str) -> Result<()> {
    let result = StdCommand::new(name)
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .output();

    match result {
        Ok(_) => Ok(()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
            bail!("`{}` not found on PATH. Please install it.", name)
        }
        Err(err) => bail!("Failed to execute `{}`: {}", name, err),
    }
}
