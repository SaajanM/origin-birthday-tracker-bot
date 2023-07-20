use chrono::{DateTime, Days, Utc};
use poise::serenity_prelude::{GuildId, UserId};

use crate::structs::{Context, Error};

/// List all birthdays on the server that have happened today (WIP)
#[poise::command(slash_command)]
pub async fn today(ctx: Context<'_>) -> Result<(), Error> {
    let mut res = "People with birthdays today:\n".to_string();
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

    let mut count = 0;

    for info in birthday_map.ordered_iter() {
        let user_str = match GuildId(guild_id)
            .member(ctx, UserId(info.associated_user))
            .await
        {
            Ok(user) => user.display_name().to_string(),
            Err(_) => "UserFetchError".to_string(),
        };

        let eod = info.datetime.checked_add_days(Days::new(1));
        let eod = match eod {
            Some(eod) => DateTime::with_timezone(&eod, &Utc),
            None => {
                ctx.say("Could not calculate EOD for timekeeping measures")
                    .await?;
                return Ok(());
            }
        };

        if !(info.datetime < Utc::now() && Utc::now() < eod) {
            continue;
        }

        res += format!("- {}\n", user_str,).as_str();

        count += 1;
    }
    if count > 0 {
        ctx.say(res).await?;
    } else {
        ctx.say("None of the registered birthdays are today.")
            .await?;
    }

    Ok(())
}
