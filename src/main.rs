use anyhow::Result;
use clap::{Parser, Subcommand};

mod config;

#[derive(Debug, Parser)]
#[command(
    name = "ttv",
    version,
    about = "Watch Twitch streams via streamlink and mpv"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Debug, Subcommand)]
enum Commands {
    Config(config::ConfigArgs),
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Config(args) => config::run(args),
    }
}
