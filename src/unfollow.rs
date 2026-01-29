use std::collections::HashSet;

use anyhow::Result;
use clap::Args;

use crate::db;

#[derive(Debug, Args)]
#[command(about = "Unfollow Twitch streamers locally")]
pub struct UnfollowArgs {
    #[arg(value_name = "LOGIN", required = true, num_args = 1.., help = "Twitch login name(s) to unfollow")]
    pub logins: Vec<String>,
}

pub async fn run(args: UnfollowArgs) -> Result<()> {
    let pool = db::connect().await?;

    let mut removed = 0u64;
    let mut missing = Vec::new();
    let mut seen = HashSet::new();
    for login in &args.logins {
        let key = login.to_lowercase();
        if !seen.insert(key) {
            continue;
        }

        let affected = db::delete_streamer_by_login(&pool, login).await?;
        if affected == 0 {
            missing.push(login.clone());
        } else {
            removed += affected;
        }
    }

    if !missing.is_empty() {
        eprintln!("Not followed: {}", missing.join(", "));
    }

    println!("Unfollowed {} streamer(s).", removed);
    Ok(())
}
