use crate::origin_bot::{Context, Error};
use chrono_tz::Tz;
use std::str::FromStr;

/// Sets the server default timezone
#[poise::command(slash_command)]
pub async fn timezone(
    ctx: Context<'_>,
    #[description = "The timezone to set the server default to"] timezone_str: String,
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

    let mut guild_data_mut = data.guild_map.write().await;
    let guild_entry = guild_data_mut.entry(guild_id).or_default();

    // Parse Timezone
    let timezone: Result<Tz, _> = Tz::from_str(&timezone_str);

    let timezone_to_set: Tz = match timezone {
        Ok(tz) => tz,
        Err(e) => {
            ctx.say(format!("Invalid Timezone: due to {}", e)).await?;
            return Ok(());
        }
    };

    let mut guild_entry_mut = guild_entry.write().await;
    guild_entry_mut.timezone = Some(timezone_to_set);

    ctx.say("Default timezone set successfully!").await?;

    Ok(())
}
