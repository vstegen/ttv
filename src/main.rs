use anyhow::Result;
use clap::{Parser, Subcommand};

mod auth;
mod config;

#[derive(Debug, Parser)]
#[command(
    name = "ttv",
    version,
    about = "Watch Twitch streams via streamlink and mpv",
    long_about = "ttv is a small CLI for interacting with Twitch. It manages API credentials and will provide commands to follow, list, and watch streams."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Config(config::ConfigArgs),
    Auth(auth::AuthArgs),
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Config(args) => config::run(args),
        Commands::Auth(args) => auth::run(args).await,
    }
}
