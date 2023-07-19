use poise::serenity_prelude::{GuildId, UserId};

use crate::structs::{Context, Error};

/// List all birthdays on the server
#[poise::command(slash_command)]
pub async fn list(ctx: Context<'_>) -> Result<(), Error> {
    let mut res = "Birthdays:\n".to_string();
    let guild_id = match ctx.guild_id() {
        Some(id) => id.0,
        None => {
            ctx.say("Only works inside servers (sorry lonely loser)")
                .await?;
            return Ok(());
        }
    };
    let reader = ctx.data().state.guild_map.read().await;
    let data = match reader.get(&guild_id) {
        Some(data) => data.rw_lock.read().await,
        None => {
            ctx.say("This server has no birthdays").await?;
            return Ok(());
        }
    };

    let birthday_map = &data.birthday_schedule;

    if birthday_map.is_empty() {
        ctx.say("This server has no birthdays").await?;
        return Ok(());
    }

    let mut bold_char = "**";
    let mut postfix = " (nearest birthday)";

    for info in birthday_map.ordered_iter() {
        let user_str = match GuildId(guild_id)
            .member(ctx, UserId(info.associated_user))
            .await
        {
            Ok(user) => user.display_name().to_string(),
            Err(_) => "UserFetchError".to_string(),
        };

        res += format!(
            "- {b}{}'s birthday is on {}{b}{p}\n",
            user_str,
            info.datetime.format("%B %e"),
            b = bold_char,
            p = postfix
        )
        .as_str();

        bold_char = "";
        postfix = "";
    }
    ctx.say(res).await?;
    Ok(())
}
