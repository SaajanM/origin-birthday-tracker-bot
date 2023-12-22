use crate::{
    helpers::is_guild_setup,
    persistence::{CommandWithCallback, DbCommand},
    structs::{Context, Error},
};
use tokio::sync::oneshot;

/// Remove a birthday from the list
#[poise::command(slash_command)]
pub async fn remove(
    ctx: Context<'_>,
    #[description = "Entry to remove"] entry_name: String,
) -> Result<(), Error> {
    let query_handler = &ctx.data().query_handler;

    let guild_id = match ctx.guild_id() {
        Some(id) => id.0,
        None => ctx.channel_id().0,
    };

    match is_guild_setup(query_handler, guild_id).await {
        Err(err_str) => {
            let _ = ctx.say(err_str).await;
            return Ok(());
        }
        Ok(false) => {
            let _ = ctx.say("Server is not set up").await;
            return Ok(());
        }
        _ => {}
    };

    let (callback, callback_recv) = oneshot::channel();

    let query_send_result = query_handler.send(DbCommand::RemoveBirthday(CommandWithCallback {
        data: (guild_id, entry_name),
        callback,
    }));

    if query_send_result.is_err() {
        ctx.say("Cannot connect to data store").await?;
        return Ok(());
    }

    match callback_recv.await {
        Ok(Ok(_)) => {
            ctx.say("Successfully removed birthday").await?;
            Ok(())
        }
        Ok(Err(_)) => {
            ctx.say("Issue removing birthday from the data store. Maybe a birthday with this entry name does not exist?").await?;
            Ok(())
        }
        Err(_) => {
            ctx.say("Could not recieve message from data store").await?;
            Ok(())
        }
    }
}
