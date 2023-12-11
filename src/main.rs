use clap::Parser;
use origin_bot::start_bot;
use serde::Deserialize;
use serenity::prelude::*;

pub mod commands;
pub mod cron;
pub mod models;
mod origin_bot;
pub mod persistence;
pub mod schema;
pub mod structs;

#[derive(Deserialize)]
struct DiscordBotEnv {
    pub discord_token: String,
    #[serde(rename = "database_url")]
    pub save_location: String,
}

#[derive(Parser)]
#[command(author,version, about, long_about = None)]
struct Args {}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let _args = Args::parse();

    let env_config: DiscordBotEnv = envy::from_env()?;

    let token = env_config.discord_token;

    let intents = GatewayIntents::default();

    match start_bot(token, intents, env_config.save_location).await {
        Ok(_) => Ok(()),
        Err(e) => Err(anyhow::anyhow!("Serenity Error: {}", e)),
    }
}
