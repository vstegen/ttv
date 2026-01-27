use std::collections::HashSet;

use anyhow::{bail, Result};
use chrono::Utc;
use clap::Args;

use crate::{auth, config, db, twitch};

#[derive(Debug, Args)]
#[command(about = "Follow Twitch streamers locally")]
pub struct FollowArgs {
    #[arg(value_name = "LOGIN", required = true, num_args = 1.., help = "Twitch login name(s) to follow")]
    pub logins: Vec<String>,
}

pub async fn run(args: FollowArgs) -> Result<()> {
    let mut config = config::load_config()?;
    if token_needs_refresh(&config) {
        auth::run(auth::AuthArgs {
            show: false,
            verbose: false,
        })
        .await?;
        config = config::load_config()?;
    }

    let client_id = config
        .twitch
        .client_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            anyhow::anyhow!("Missing Twitch client ID. Run `ttv config --client-id <ID>`.")
        })?;

    let access_token = config
        .twitch
        .access_token
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| anyhow::anyhow!("Missing Twitch access token. Run `ttv auth`."))?;

    let users = twitch::fetch_users_by_login(client_id, access_token, &args.logins).await?;
    if users.is_empty() {
        bail!("No streamers found for the provided login names.");
    }

    let pool = db::connect().await?;
    for user in &users {
        db::upsert_streamer(&pool, user).await?;
    }

    let found: HashSet<String> = users
        .iter()
        .map(|user| user.login.to_lowercase())
        .collect();
    let missing: Vec<String> = args
        .logins
        .iter()
        .filter(|login| !found.contains(&login.to_lowercase()))
        .cloned()
        .collect();

    if !missing.is_empty() {
        eprintln!("Not found on Twitch: {}", missing.join(", "));
    }

    println!("Followed {} streamer(s).", users.len());
    Ok(())
}

fn token_needs_refresh(config: &config::Config) -> bool {
    let token = config
        .twitch
        .access_token
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let expires_at = config.twitch.expires_at;

    match (token, expires_at) {
        (Some(_), Some(expires_at)) => Utc::now() >= expires_at,
        _ => true,
    }
}
