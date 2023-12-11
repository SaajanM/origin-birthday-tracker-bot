use crate::structs::{Context, Error};

/// Get the current ping to the bot
#[poise::command(slash_command, owners_only)]
pub async fn ping(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say(format!("Current ping: {}ms", ctx.ping().await.as_millis()))
        .await?;

    Ok(())
}
