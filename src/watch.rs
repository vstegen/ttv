use std::collections::HashSet;
use anyhow::{bail, Context, Result};
use clap::Args;
use crate::streamlink;

#[derive(Debug, Args)]
#[command(about = "Watch Twitch streams via streamlink and mpv")]
pub struct WatchArgs {
    #[arg(value_name = "STREAM", required = true, num_args = 1.., help = "Twitch login or URL")]
    pub streams: Vec<String>,
}

pub async fn run(args: WatchArgs) -> Result<()> {
    streamlink::ensure_dependencies()?;

    let logins = normalize_inputs(&args.streams)?;
    if logins.is_empty() {
        bail!("No valid Twitch streams provided.");
    }

    let mut handles = Vec::new();
    for login in logins {
        let url = format!("https://www.twitch.tv/{login}");
        println!("Starting stream for {login}...");

        let mut child = streamlink::spawn(&url)
            .with_context(|| format!("failed to start streamlink for {login}"))?;

        handles.push(tokio::spawn(async move {
            let status = child.wait().await;
            (login, status)
        }));
    }

    let mut failed = Vec::new();
    for handle in handles {
        let (login, status) = handle.await.context("failed to join stream task")?;
        match status {
            Ok(exit) if exit.success() => {}
            Ok(exit) => failed.push(format!("{login} (exit {exit})")),
            Err(err) => failed.push(format!("{login} ({err})")),
        }
    }

    if !failed.is_empty() {
        bail!(
            "Some streams failed to start or exited early: {}",
            failed.join(", ")
        );
    }

    Ok(())
}

fn normalize_inputs(inputs: &[String]) -> Result<Vec<String>> {
    let mut seen = HashSet::new();
    let mut logins = Vec::new();

    for input in inputs {
        let login = parse_login(input)?;
        let key = login.to_lowercase();
        if seen.insert(key.clone()) {
            logins.push(key);
        }
    }

    Ok(logins)
}

fn parse_login(input: &str) -> Result<String> {
    if let Some(login) = parse_twitch_url(input) {
        return Ok(login);
    }

    if is_valid_login(input) {
        return Ok(input.to_string());
    }

    bail!("Invalid Twitch URL or login: {input}")
}

fn parse_twitch_url(input: &str) -> Option<String> {
    let without_scheme = input
        .strip_prefix("https://")
        .or_else(|| input.strip_prefix("http://"))?;
    let without_www = without_scheme
        .strip_prefix("www.")
        .unwrap_or(without_scheme);
    let path = without_www.strip_prefix("twitch.tv/")?;

    if path.is_empty() || path.contains('/') || path.contains('?') || path.contains('#') {
        return None;
    }

    if !is_valid_login(path) {
        return None;
    }

    Some(path.to_string())
}

fn is_valid_login(login: &str) -> bool {
    !login.is_empty()
        && login
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}
