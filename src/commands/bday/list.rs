use crate::{
    helpers::{get_disp_name_from_entry_name, is_guild_setup},
    persistence::{CommandWithCallback, DbCommand},
    structs::{Context, Error},
};
use tokio::sync::oneshot;

/// List out the upcoming birthdays
#[poise::command(slash_command)]
pub async fn list(
    ctx: Context<'_>,
    #[description = "Maximum number of birthdays to list (default: 20)"] limit: Option<i32>,
) -> Result<(), Error> {
    ctx.defer().await?;

    let query_handler = &ctx.data().query_handler;

    let limit = limit.unwrap_or(20);

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

    let query_result = query_handler.send(DbCommand::FetchNBirthdays(CommandWithCallback {
        data: (guild_id, limit),
        callback,
    }));

    if query_result.is_err() {
        ctx.say("Listing failed: Could not connect to data store")
            .await?;
        return Ok(());
    }

    let (birthdays, remaining_entries) = match callback_recv.await {
        Ok(Ok(v)) => v,
        Ok(Err(_)) => {
            ctx.say("Listing failed: Could not retrieve birthdays from data store")
                .await?;
            return Ok(());
        }
        Err(_) => {
            ctx.say("Listing failed: Could not connect to data store")
                .await?;
            return Ok(());
        }
    };

    if birthdays.is_empty() {
        ctx.say("No birthdays set up in this server").await?;
        return Ok(());
    }

    let mut resp_string = "Nearest birthdays:\n".to_owned();

    let guild_opt = ctx.guild_id();

    for birthday in birthdays {
        let format_tag = match birthday.uses_time {
            true => "",
            false => ":D",
        };

        let entry_name =
            get_disp_name_from_entry_name(birthday.entry_name.clone(), ctx, guild_opt).await;

        resp_string += format!(
            "- {}'s next birthday is at <t:{}{}>\n",
            entry_name, birthday.next_birthday, format_tag
        )
        .as_str();
    }

    if remaining_entries > 0 {
        resp_string += format!("... and {} more", remaining_entries).as_str();
    }

    ctx.say(resp_string).await?;

    Ok(())
}
