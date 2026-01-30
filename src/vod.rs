use std::io::{self, Write};

use anyhow::{bail, Context, Result};
use clap::Args;

use crate::{auth, config, streamlink, twitch};

#[derive(Debug, Args)]
#[command(about = "Watch VODs for a Twitch streamer")]
pub struct VodArgs {
    #[arg(value_name = "LOGIN", help = "Twitch login name")]
    pub login: String,
}

pub async fn run(args: VodArgs) -> Result<()> {
    streamlink::ensure_dependencies()?;

    let mut config = config::load_config()?;
    if config::token_needs_refresh(&config) {
        auth::run(auth::AuthArgs {
            show: false,
            verbose: false,
        })
        .await?;
        config = config::load_config()?;
    }

    let client_id = config::require_client_id(&config)?;
    let access_token = config::require_access_token(&config)?;

    let user = twitch::fetch_user_by_login(client_id, access_token, &args.login).await?;
    let vods = twitch::fetch_vods_by_user_id(client_id, access_token, &user.id).await?;

    if vods.is_empty() {
        println!("No VODs found for {}.", user.display_name);
        return Ok(());
    }

    println!("VODs for {}:", user.display_name);
    for (idx, vod) in vods.iter().enumerate() {
        println!(
            "{:>2}) [{}] {} ({})",
            idx + 1,
            vod.created_at,
            vod.title,
            vod.duration
        );
    }

    let selection = prompt_selection(vods.len())?;
    let vod = &vods[selection - 1];
    let url = format!("https://www.twitch.tv/videos/{}", vod.id);
    println!("Starting VOD {}...", vod.id);

    streamlink::launch(&url).await?;

    Ok(())
}

fn prompt_selection(max: usize) -> Result<usize> {
    loop {
        print!("Select a VOD (1-{}): ", max);
        io::stdout().flush().ok();

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .context("failed to read input")?;

        let trimmed = input.trim();
        if trimmed.is_empty() {
            bail!("No selection provided.");
        }

        match trimmed.parse::<usize>() {
            Ok(value) if (1..=max).contains(&value) => return Ok(value),
            _ => println!(
                "Invalid selection. Please enter a number between 1 and {}.",
                max
            ),
        }
    }
}
