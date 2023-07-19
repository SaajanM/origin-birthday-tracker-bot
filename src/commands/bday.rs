use crate::structs::{BirthdayInfo, Context, Error};
use chrono::{DateTime, Datelike, Days, FixedOffset, Offset};
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
            let read = guild_entry.rw_lock.read().await;
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

    let (mut datetime, eod) = match timezone {
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
                Ok(datetime) => {
                    let eod = datetime.checked_add_days(Days::new(1));
                    let eod = match eod {
                        Some(eod) => DateTime::with_timezone(&eod, &Utc),
                        None => {
                            ctx.say("Could not calculate EOD for timekeeping measures")
                                .await?;
                            return Ok(());
                        }
                    };
                    (DateTime::with_timezone(&datetime, &Utc), eod)
                }
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
    let now = Utc::now();
    if datetime < now && now >= eod {
        //Just in case the bday already happened (and fail back if need be)
        datetime = match datetime.with_year(datetime.year() + 1) {
            Some(new_dt) => new_dt,
            None => datetime,
        };
    }

    let new_entry = Arc::new(BirthdayInfo {
        associated_user: user.user.id.0,
        datetime,
    });

    let mut guild_data_write = guild_entry.rw_lock.write().await;
    let _ = guild_data_write
        .birthday_schedule
        .insert(Arc::clone(&new_entry));

    ctx.data().saver.save();

    ctx.say(format!(
        "Adding birthday for {} on {}",
        user.display_name(),
        datetime
    ))
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
