use crate::structs::{BirthdayInfo, Context, Error};
use chrono::{DateTime, Datelike, Days, FixedOffset, Offset};
use chrono::{TimeZone, Utc};
use chrono_tz::Tz;
use poise::serenity_prelude::{self as serenity};
use std::str::FromStr;
use std::sync::Arc;

/// Set a birthday for a user
#[poise::command(slash_command)]
pub async fn set(
    ctx: Context<'_>,
    #[description = "The User you are adding a birthday for"] user: Option<serenity::Member>,
    #[description = "The day (MM/DD format) on which to give a birthday announcement"]
    day_str: String,
    #[description = "The timezone (Region/Location format) to use (if not provided, server default is used)."]
    timezone_str: Option<String>,
) -> Result<(), Error> {
    let user = user.unwrap_or(match ctx.author_member().await {
        Some(user) => user.into_owned(),
        None => {
            ctx.say("Only works inside servers (sorry lonely loser)")
                .await?;
            return Ok(());
        }
    });
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
