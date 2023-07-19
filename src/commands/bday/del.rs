use poise::serenity_prelude::Member;

use crate::structs::{Context, Error};

/// Delete a users birthday
#[poise::command(slash_command)]
pub async fn del(
    ctx: Context<'_>,
    #[description = "The User you are deleting a birthday for"] user: Member,
) -> Result<(), Error> {
    let guild_id = match ctx.guild_id() {
        Some(id) => id.0,
        None => {
            ctx.say("Only works inside servers (sorry lonely loser)")
                .await?;
            return Ok(());
        }
    };
    let data = ctx.data().state.guild_map.read().await;
    match data.get(&guild_id) {
        Some(guild_data) => {
            let mut guild_writer = guild_data.rw_lock.write().await;

            let deletion = guild_writer.birthday_schedule.remove(&user.user.id.0);

            if deletion.is_some() {
                ctx.data().saver.save();
                ctx.say("Birthday removed successfully").await?;
            } else {
                ctx.say("User is not registered with the birthday service")
                    .await?;
            }

            Ok(())
        }
        None => {
            ctx.say("No birthdays found in this server").await?;
            Ok(())
        }
    }
}
