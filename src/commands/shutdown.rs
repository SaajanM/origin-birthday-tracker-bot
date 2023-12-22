use crate::{
    persistence::DbCommand,
    structs::{Context, Error},
};

/// Shut down the bot
#[poise::command(slash_command, owners_only)]
pub async fn shutdown(ctx: Context<'_>) -> Result<(), Error> {
    let query_handler = &ctx.data().query_handler;

    let query_send_result = query_handler.send(DbCommand::Shutdown);

    match query_send_result {
        Ok(_) => ctx.say("Shutting down...").await?,
        Err(_) => ctx.say("Failed to initiate shutdown").await?,
    };

    Ok(())
}
