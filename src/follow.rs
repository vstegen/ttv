use std::collections::HashSet;

use anyhow::{Result, bail};
use clap::Args;

use crate::{auth, config, db, twitch};

#[derive(Debug, Args)]
#[command(about = "Follow Twitch streamers locally")]
pub struct FollowArgs {
    #[arg(value_name = "LOGIN", required = true, num_args = 1.., help = "Twitch login name(s) to follow")]
    pub logins: Vec<String>,
    #[arg(long, help = "Print verbose request and update details")]
    pub verbose: bool,
}

pub async fn run(args: FollowArgs) -> Result<()> {
    let mut config = config::load_config()?;
    if config::token_needs_refresh(&config) {
        if args.verbose {
            eprintln!("[INFO] Access token missing or expired, running auth");
        }
        auth::run(auth::AuthArgs {
            show: false,
            verbose: args.verbose,
        })
        .await?;
        config = config::load_config()?;
    }

    let client_id = config::require_client_id(&config)?;
    let access_token = config::require_access_token(&config)?;

    if args.verbose {
        eprintln!("[INFO] Fetching {} streamer(s) from Twitch", args.logins.len());
    }
    let users = twitch::fetch_users_by_login(client_id, access_token, &args.logins).await?;
    if users.is_empty() {
        bail!("No streamers found for the provided login names.");
    }

    let pool = db::connect().await?;
    if args.verbose {
        if let Ok(path) = db::db_path() {
            eprintln!("[INFO] Using database at {}", path.display());
        }
    }
    for user in &users {
        db::upsert_streamer(&pool, user).await?;
        if args.verbose {
            eprintln!(
                "[INFO] Followed {} ({})",
                user.login, user.display_name
            );
        }
    }

    let found: HashSet<String> = users.iter().map(|user| user.login.to_lowercase()).collect();
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
