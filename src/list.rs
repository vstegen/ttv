use std::collections::HashMap;

use anyhow::Result;
use clap::{Args, ValueEnum};

use crate::{auth, config, db, twitch};

#[derive(Debug, Clone, ValueEnum)]
pub enum ListStatus {
    Online,
    Offline,
    All,
}

#[derive(Debug, Args)]
#[command(about = "List followed streamers")]
pub struct ListArgs {
    #[arg(long, value_enum, default_value_t = ListStatus::Online, help = "Filter by online status")]
    pub status: ListStatus,
}

pub async fn run(args: ListArgs) -> Result<()> {
    let pool = db::connect().await?;
    let streamers = db::list_streamers(&pool).await?;
    if streamers.is_empty() {
        println!("No followed streamers.");
        return Ok(());
    }

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

    let ids: Vec<String> = streamers
        .iter()
        .map(|streamer| streamer.id.clone())
        .collect();
    let streams = twitch::fetch_streams_by_user_ids(client_id, access_token, &ids).await?;
    let online_map: HashMap<String, twitch::TwitchStream> = streams
        .into_iter()
        .map(|stream| (stream.user_id.clone(), stream))
        .collect();

    let mut rows = Vec::new();
    for streamer in &streamers {
        let online = online_map.get(&streamer.id);
        match args.status {
            ListStatus::Online => {
                if let Some(stream) = online {
                    rows.push(Row::online(streamer, stream));
                }
            }
            ListStatus::Offline => {
                if online.is_none() {
                    rows.push(Row::offline(streamer));
                }
            }
            ListStatus::All => {
                rows.push(match online {
                    Some(stream) => Row::online_with_status(streamer, stream),
                    None => Row::offline_with_status(streamer),
                });
            }
        }
    }

    if rows.is_empty() {
        match args.status {
            ListStatus::Online => println!("No online streamers."),
            ListStatus::Offline => println!("No offline streamers."),
            ListStatus::All => println!("No streamers found."),
        }
        return Ok(());
    }

    print_table(&rows, matches!(args.status, ListStatus::All));
    Ok(())
}

struct Row {
    login: String,
    display_name: String,
    game_name: String,
    status: Option<&'static str>,
}

impl Row {
    fn online(streamer: &db::DbStreamer, stream: &twitch::TwitchStream) -> Self {
        Self {
            login: streamer.name.clone(),
            display_name: streamer.display_name.clone(),
            game_name: stream.game_name.clone(),
            status: None,
        }
    }

    fn offline(streamer: &db::DbStreamer) -> Self {
        Self {
            login: streamer.name.clone(),
            display_name: streamer.display_name.clone(),
            game_name: String::new(),
            status: None,
        }
    }

    fn online_with_status(streamer: &db::DbStreamer, stream: &twitch::TwitchStream) -> Self {
        Self {
            login: streamer.name.clone(),
            display_name: streamer.display_name.clone(),
            game_name: stream.game_name.clone(),
            status: Some("online"),
        }
    }

    fn offline_with_status(streamer: &db::DbStreamer) -> Self {
        Self {
            login: streamer.name.clone(),
            display_name: streamer.display_name.clone(),
            game_name: String::new(),
            status: Some("offline"),
        }
    }
}

fn print_table(rows: &[Row], include_status: bool) {
    let login_width = rows
        .iter()
        .map(|row| row.login.len())
        .max()
        .unwrap_or(5)
        .max("login".len());
    let display_width = rows
        .iter()
        .map(|row| row.display_name.len())
        .max()
        .unwrap_or(12)
        .max("display_name".len());
    let game_width = rows
        .iter()
        .map(|row| row.game_name.len())
        .max()
        .unwrap_or(4)
        .max("game".len());
    let status_width = if include_status {
        rows.iter()
            .filter_map(|row| row.status.map(|status| status.len()))
            .max()
            .unwrap_or(6)
            .max("status".len())
    } else {
        0
    };

    if include_status {
        println!(
            "{:<login_width$}  {:<display_width$}  {:<game_width$}  {:<status_width$}",
            "login",
            "display_name",
            "game",
            "status",
            login_width = login_width,
            display_width = display_width,
            game_width = game_width,
            status_width = status_width
        );
    } else {
        println!(
            "{:<login_width$}  {:<display_width$}  {:<game_width$}",
            "login",
            "display_name",
            "game",
            login_width = login_width,
            display_width = display_width,
            game_width = game_width
        );
    }

    for row in rows {
        if include_status {
            let status = row.status.unwrap_or("");
            println!(
                "{:<login_width$}  {:<display_width$}  {:<game_width$}  {:<status_width$}",
                row.login,
                row.display_name,
                row.game_name,
                status,
                login_width = login_width,
                display_width = display_width,
                game_width = game_width,
                status_width = status_width
            );
        } else {
            println!(
                "{:<login_width$}  {:<display_width$}  {:<game_width$}",
                row.login,
                row.display_name,
                row.game_name,
                login_width = login_width,
                display_width = display_width,
                game_width = game_width
            );
        }
    }
}
