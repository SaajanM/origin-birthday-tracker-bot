use crate::origin_bot::{BirthdayInfo, Context, Error};
use chrono::{DateTime, Datelike, FixedOffset, Offset};
use chrono::{TimeZone, Utc};
use chrono_tz::Tz;
use poise::serenity_prelude::{self as serenity, GuildId, UserId};
use std::str::FromStr;
use std::sync::Arc;

/// Parent Command for all birthdat relayed doodads
#[poise::command(slash_command, subcommands("set", "del", "list"))]
pub async fn bday(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Use one of the subcommands to alter this guilds birthday list")
        .await?;
    Ok(())
}

/// Set a birthday for a user
#[poise::command(slash_command)]
pub async fn set(
    ctx: Context<'_>,
    #[description = "The User you are adding a birthday for"] user: serenity::Member,
    #[description = "The day (MM/DD format) on which to give a birthday announcement"]
    day_str: String,
    #[description = "The timezone (Region/Location format) to use (if not provided, server default is used)."]
    timezone_str: Option<String>,
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
    let timezone: Result<Tz, _> = match timezone_str {
        Some(timezone_str) => Tz::from_str(&timezone_str),
        None => {
            let read = guild_entry.read().await;
            match read.timezone {
                Some(tz) => Ok(tz),
                None => {
                    ctx.say("Guild does not have a default timezone set. Please provide one")
                        .await?;
                    return Ok(());
                }
            }
        }
    };

    let datetime = match timezone {
        Ok(tz) => {
            let offset: FixedOffset = tz.offset_from_utc_datetime(&Utc::now().naive_utc()).fix();
            let seconds_offset = offset.local_minus_utc();
            let hours = seconds_offset.abs() / 3600;
            let minutes = (seconds_offset.abs() - hours * 3600) / 60;
            let offset_char = if seconds_offset > 0 { "+" } else { "-" };
            let formatted = format!(
                "{} {} 00:00:00 {}{:0>2}{:0>2}",
                Utc::now().year(),
                day_str,
                offset_char,
                hours,
                minutes
            );
            match DateTime::parse_from_str(formatted.as_str(), "%Y %m/%d %H:%M:%S %z") {
                Ok(datetime) => DateTime::with_timezone(&datetime, &Utc),
                Err(e) => {
                    ctx.say(format!("Invalid Datetime {}: due to {}", formatted, e))
                        .await?;
                    return Ok(());
                }
            }
        }
        Err(e) => {
            ctx.say(format!("Invalid Timezone: due to {}", e)).await?;
            return Ok(());
        }
    };

    let new_entry = Arc::new(BirthdayInfo {
        associated_user: user.user.id.0,
        datetime,
    });

    let mut guild_data_write = guild_entry.write().await;
    guild_data_write
        .birthday_map
        .insert(user.user.id.0, Arc::clone(&new_entry));
    guild_data_write.schedule.insert(new_entry);
    ctx.say(format!("Adding birthday with {}", datetime.timestamp()))
        .await?;

    Ok(())
}

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
        Some(data) => data.read().await,
        None => {
            ctx.say("This server has no birthdays").await?;
            return Ok(());
        }
    };

    let birthday_map = &data.birthday_map;

    if birthday_map.is_empty() {
        ctx.say("This server has no birthdays").await?;
        return Ok(());
    }

    for (id, info) in birthday_map.iter() {
        let user_str = match GuildId(guild_id).member(ctx, UserId(*id)).await {
            Ok(user) => user.display_name().to_string(),
            Err(_) => "UserFetchError".to_string(),
        };

        res += format!(
            " - {}'s birthday is on {}\n",
            user_str,
            info.datetime.format("%B %e")
        )
        .as_str();
    }
    ctx.say(res).await?;
    Ok(())
}

/// Delete a users birthday
#[poise::command(slash_command)]
pub async fn del(
    ctx: Context<'_>,
    #[description = "The User you are deleting a birthday for"] user: serenity::Member,
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
            let mut guild_writer = guild_data.write().await;

            let map_del = guild_writer.birthday_map.remove(&user.user.id.0);

            let removed = match map_del {
                Some(value) => guild_writer.schedule.remove(&value),
                None => false,
            };

            if removed {
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
