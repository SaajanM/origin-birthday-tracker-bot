use origin_bot::start_bot;
use serde::Deserialize;
use serenity::prelude::*;

pub mod commands;
pub mod cron;
mod origin_bot;

#[derive(Deserialize)]
struct DiscordBotEnv {
    pub discord_token: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv()?;

    let env_cofig: DiscordBotEnv = envy::from_env()?;

    let token = env_cofig.discord_token;

    let intents = GatewayIntents::empty();

    match start_bot(token, intents).await {
        Ok(_) => {
            println!("Starting Origin Bot...");
            Ok(())
        }
        Err(e) => Err(anyhow::anyhow!("Serenity Error: {}", e)),
    }
}
