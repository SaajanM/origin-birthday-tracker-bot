use crate::{
    helpers::{autocomplete_tz, calc_next_birthday, is_guild_setup},
    models::NewBirthday,
    persistence::{CommandWithCallback, DbCommand},
    structs::{Context, Error},
};
use chrono::{NaiveDate, NaiveTime, Utc};
use poise::serenity_prelude::User;
use serenity::utils::parse_username;
use tokio::sync::oneshot;

/// Add a birthday to the list
#[poise::command(slash_command)]
pub async fn add(
    ctx: Context<'_>,
    #[description = "Who to celebrate for"] entry_name: String,
    #[description = "The date of this birthday (as MM/DD)"] birth_date: String,
    #[description = "Optional time of birthday (as 24hr hh:mm:ss, seconds optional) (default: start of day at midnight)"]
    birth_time: Option<String>,
    #[description = "The timezone for this birthday (default: the server's default timezone)"]
    #[autocomplete = "autocomplete_tz"]
    timezone: Option<String>,
    #[description = "Who to ping for any alerts (default: caller of this command, or entry name if a discord handle)"]
    who_to_ping: Option<User>,
) -> Result<(), Error> {
    let query_handler = &ctx.data().query_handler;

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

    let birth_date = birth_date + "/2000";

    let birthdate = match NaiveDate::parse_from_str(&birth_date, "%m/%d/%Y") {
        Ok(v) => v,
        Err(_) => {
            ctx.say("Please use MM/DD/YYYY syntax for the date").await?;
            return Ok(());
        }
    };

    let mut uses_time = false;
    let birth_time = match birth_time {
        Some(b) => {
            uses_time = true;
            b
        }
        None => "00:00".to_owned(),
    };
    let birth_time: NaiveTime = match birth_time.parse() {
        Ok(v) => v,
        Err(_) => {
            ctx.say("Please use hh:mm:ss syntax for the time (seconds optional)")
                .await?;
            return Ok(());
        }
    };

    // let who_to_ping = who_to_ping.map(|x| x.id.0).unwrap_or(ctx.author().id.0) as i64;
    let who_to_ping = match who_to_ping {
        Some(user) => user.id.0,
        None => match parse_username(entry_name.clone()) {
            Some(user_id) => user_id,
            None => ctx.author().id.0,
        },
    } as i64;

    let timezone = match timezone {
        Some(tz) => tz,
        None => {
            let (callback, callback_recv) = oneshot::channel();

            let query_send_result =
                query_handler.send(DbCommand::GetGuildData(CommandWithCallback {
                    data: guild_id,
                    callback,
                }));

            if query_send_result.is_err() {
                ctx.say("Cannot connect to data store").await?;
                return Ok(());
            }

            match callback_recv.await {
                Ok(Ok(guild)) => match guild.timezone_name {
                    Some(tz) => tz,
                    None => {
                        ctx.say("No timezone provided and server has no default timezone")
                            .await?;
                        return Ok(());
                    }
                },
                Ok(Err(_)) => {
                    ctx.say("Could not fetch server timezone from data store")
                        .await?;
                    return Ok(());
                }
                Err(e) => {
                    println!("{}", e);
                    ctx.say("Could not recieve data from data store").await?;
                    return Ok(());
                }
            }
        }
    };
    let timezone = match chrono_tz::Tz::from_str_insensitive(&timezone) {
        Ok(v) => v,
        Err(_) => {
            ctx.say("Please enter a valid timezone").await?;
            return Ok(());
        }
    };

    let birthday = match birthdate.and_time(birth_time).and_local_timezone(timezone) {
        chrono::LocalResult::Single(v) => v.with_timezone(&Utc),
        _ => {
            ctx.say(
                "Using this date & time with this timezone yields an invalid or ambiguous time",
            )
            .await?;
            return Ok(());
        }
    };

    let next_birthday = calc_next_birthday(birthday);

    // let birthday = birthday.timestamp();

    let command_data = NewBirthday {
        next_birthday,
        uses_time,
        who_to_ping,
        entry_name,
        guild_id: guild_id as i64,
    };

    let (callback, callback_recv) = oneshot::channel();

    let query_send_result = query_handler.send(DbCommand::AddBirthday(CommandWithCallback {
        data: command_data,
        callback,
    }));

    if query_send_result.is_err() {
        ctx.say("Cannot connect to data store").await?;
        return Ok(());
    }

    match callback_recv.await {
        Ok(Ok(birthday)) => {
            ctx.say(format!(
                "Successfully added birthday. Next ping <t:{}:R>",
                birthday.next_birthday as u64
            ))
            .await?;
            Ok(())
        }
        Ok(Err(_)) => {
            ctx.say("Issue adding date to data store. Maybe a birthday with this entry name already exists?").await?;
            Ok(())
        }
        Err(_) => {
            ctx.say("Could not recieve message from data store").await?;
            Ok(())
        }
    }
}
