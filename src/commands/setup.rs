use crate::{
    models::NewGuild,
    persistence::{CommandWithCallback, DbCommand},
    structs::{Context, Error},
};
use chrono_tz::Tz;
use poise::serenity_prelude::Channel;
use tokio::sync::oneshot;

/// Set up this guild/dm for the bot
#[poise::command(
    slash_command,
    default_member_permissions = "ADMINISTRATOR | MANAGE_GUILD"
)]
pub async fn setup(
    ctx: Context<'_>,
    #[description = "The channel that OriginBot will talk in. Leave blank if in DM"]
    announcement_channel: Option<Channel>,
    #[description = "The timezone to set the server default to"] timezone_str_opt: Option<String>,
    #[description = "Allow anyone to edit and set birthdays on this server (default: false)"]
    allows_anyone_edit_opt: Option<bool>,
    #[description = "Ping the @everyone role when the server birthday comes around (default: false)"]
    do_server_bday_opt: Option<bool>,
) -> Result<(), Error> {
    let query_handler = &ctx.data().query_handler;

    let guild_id = match ctx.guild_id() {
        Some(id) => id.0,
        None => ctx.channel_id().0,
    };

    let (callback, callback_recv) = oneshot::channel();

    let count_send_result =
        query_handler.send(DbCommand::CheckContainsGuild(CommandWithCallback {
            data: guild_id,
            callback,
        }));

    if count_send_result.is_err() {
        ctx.say("Setup failed: Could not connect to data store")
            .await?;
        return Ok(());
    }

    match callback_recv.await {
        Ok(Ok(true)) => {
            ctx.say("Setup failed: Already set up. If you want to edit this server/chat's settings please run the corresponding specific command").await?;
            return Ok(());
        }
        Err(e) => {
            println!("{}", e);
            ctx.say("Setup failed: Could not connect to data store")
                .await?;
            return Ok(());
        }
        _ => {}
    };
    let announcement_channel_id = match announcement_channel {
        Some(channel) => channel.id().0,
        None => {
            match ctx.guild() {
                Some(_) => {
                    ctx.say("Setup failed: You are in a server and must pass in an announcements channel")
                .await?;
                    return Ok(());
                }
                None => ctx.channel_id().0,
            }
        }
    };
    let allows_anyone_edit = allows_anyone_edit_opt.unwrap_or(false);
    let do_server_birthday = do_server_bday_opt.unwrap_or(false);

    if let Some(tz_str) = &timezone_str_opt {
        let is_tz_result = Tz::from_str_insensitive(tz_str.as_str());
        if is_tz_result.is_err() {
            ctx.say("Setup failed: Invalid timezone provided").await?;
            return Ok(());
        }
    }

    let new_guild = NewGuild {
        guild_id: guild_id as i64,
        announcement_channel: announcement_channel_id as i64,
        allows_anyone_edit,
        do_server_birthday,
        timezone_name: timezone_str_opt,
    };

    let (callback, callback_recv) = oneshot::channel();

    let query_send_result = query_handler.send(DbCommand::AddGuild(CommandWithCallback {
        data: new_guild,
        callback,
    }));

    if query_send_result.is_err() {
        ctx.say("Setup failed: Could not connect to data store")
            .await?;
        return Ok(());
    }

    match callback_recv.await {
        Ok(Ok(_)) => ctx.say("Setup successful!").await?,
        Ok(Err(_)) => {
            ctx.say("Setup may have failed: Data store returned error")
                .await?
        }
        Err(_) => {
            ctx.say("Setup may have failed: Could not recieve callback from data store")
                .await?
        }
    };

    Ok(())
}
