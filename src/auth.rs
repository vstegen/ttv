use anyhow::{Context, Result, bail};
use chrono::{Duration, Utc};
use clap::Args;
use reqwest::StatusCode;
use serde::Deserialize;
use std::time::Instant;

use crate::config::{self, Config};

#[derive(Debug, Args)]
#[command(about = "Fetch a new Twitch app access token and update config")]
pub struct AuthArgs {
    #[arg(long, help = "Print the updated configuration (secrets masked)")]
    pub show: bool,
    #[arg(long, help = "Print verbose request and update details")]
    pub verbose: bool,
}

#[derive(Debug, Deserialize)]
struct TokenResponse {
    access_token: String,
    expires_in: i64,
    #[allow(dead_code)]
    token_type: String,
}

pub async fn run(args: AuthArgs) -> Result<()> {
    let mut config = config::load_config()?;
    let (client_id, client_secret) = credentials(&config)?;

    let client = reqwest::Client::new();
    let params = [
        ("client_id", client_id),
        ("client_secret", client_secret),
        ("grant_type", "client_credentials"),
    ];

    if args.verbose {
        eprintln!("[INFO] POST https://id.twitch.tv/oauth2/token");
    }

    let start = Instant::now();
    let res = client
        .post("https://id.twitch.tv/oauth2/token")
        .form(&params)
        .send()
        .await
        .context("failed to send auth request to Twitch")?;

    let status = res.status();
    if args.verbose {
        eprintln!("[INFO] Response status: {}", status);
        eprintln!("[INFO] Request duration: {}ms", start.elapsed().as_millis());
    }
    if !status.is_success() {
        let body = res.text().await.unwrap_or_default();
        return Err(map_auth_error(status, body));
    }

    let token: TokenResponse = res
        .json()
        .await
        .context("failed to parse Twitch token response")?;

    let expires_at = Utc::now() + Duration::seconds(token.expires_in);
    config.twitch.access_token = Some(token.access_token);
    config.twitch.expires_at = Some(expires_at);

    config::save_config_default(&config)?;
    if args.verbose {
        if let Ok(path) = config::config_path() {
            eprintln!("[INFO] Updated config at {}", path.display());
        }
        eprintln!("[INFO] Token expires at {}", expires_at.to_rfc3339());
    }
    println!(
        "Fetched new access token (expires in {}s).",
        token.expires_in
    );
    if args.show {
        config::print_config(&config)?;
    }
    Ok(())
}

fn credentials(config: &Config) -> Result<(&str, &str)> {
    let mut missing = Vec::new();

    let client_id = config
        .twitch
        .client_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if client_id.is_none() {
        missing.push("client ID");
    }

    let client_secret = config
        .twitch
        .client_secret
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    if client_secret.is_none() {
        missing.push("client secret");
    }

    if !missing.is_empty() {
        bail!(
            "Missing Twitch {}. Run `ttv config --client-id <ID> --client-secret <SECRET>` first.",
            missing.join(" and ")
        );
    }

    Ok((client_id.unwrap(), client_secret.unwrap()))
}

fn map_auth_error(status: StatusCode, body: String) -> anyhow::Error {
    match status {
        StatusCode::BAD_REQUEST => {
            anyhow::anyhow!("Invalid Twitch client ID. Double-check `ttv config --client-id`.")
        }
        StatusCode::FORBIDDEN => anyhow::anyhow!(
            "Invalid Twitch client secret. Double-check `ttv config --client-secret`."
        ),
        StatusCode::TOO_MANY_REQUESTS => {
            anyhow::anyhow!("Twitch API rate limit exceeded. Try again later.")
        }
        _ => anyhow::anyhow!("Unexpected Twitch auth response ({}). {}", status, body),
    }
}
