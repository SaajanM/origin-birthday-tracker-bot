use crate::structs::{Context, Error};
use poise::serenity_prelude::{self as serenity};

/// Set a birthday for a user
#[poise::command(slash_command)]
pub async fn get(
    ctx: Context<'_>,
    #[description = "The User you are getting the birthday of"] user: serenity::Member,
) -> Result<(), Error> {
    let data = &ctx.data().state;

    let guild_id = match ctx.guild_id() {
        Some(id) => id.0,
        None => {
            ctx.say("Only works inside servers (sorry lonely loser)")
                .await?;
            return Ok(());
        }
    };

    let reader = data.guild_map.read().await;

    let data = match reader.get(&guild_id) {
        Some(data) => data,
        None => {
            let _ = ctx
                .say("Sorry, this server does not have any registered birthdays")
                .await;
            return Ok(());
        }
    };

    let inner_reader = data.rw_lock.read().await;

    match inner_reader.birthday_schedule.get(user.user.id.0) {
        Some(info) => {
            let _ = ctx
                .say(format!(
                    "{}'s next birthday is on {}",
                    user.display_name(),
                    info.datetime,
                ))
                .await;
        }
        None => {
            let _ = ctx
                .say(format!(
                    "{} does not seem to have a registered birthday here",
                    user.display_name(),
                ))
                .await;
        }
    }

    Ok(())
}
