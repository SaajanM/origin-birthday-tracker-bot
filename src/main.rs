use std::path::PathBuf;

use clap::Parser;
use origin_bot::start_bot;
use serde::Deserialize;
use serenity::prelude::*;

pub mod commands;
pub mod cron;
mod origin_bot;
pub mod persistence;
pub mod structs;

#[derive(Deserialize)]
struct DiscordBotEnv {
    pub discord_token: String,
}

#[derive(Parser)]
#[command(author,version, about, long_about = None)]
struct Args {
    /// Location to save and load from
    #[arg(short, long)]
    save_location: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let args = Args::parse();

    let env_cofig: DiscordBotEnv = envy::from_env()?;

    let token = env_cofig.discord_token;

    let intents = GatewayIntents::empty();

    match start_bot(token, intents, args.save_location).await {
        Ok(_) => {
            println!("Starting Origin Bot...");
            Ok(())
        }
        Err(e) => Err(anyhow::anyhow!("Serenity Error: {}", e)),
    }
}
