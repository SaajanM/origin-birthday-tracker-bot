use chrono::{DateTime, Utc};
use poise::{
    serenity_prelude::{GuildId, UserId},
    AutocompleteChoice,
};
use serenity::{http::CacheHttp, utils::parse_username};
use tokio::sync::{mpsc::UnboundedSender, oneshot};
use tzdb::TZ_NAMES;

use crate::{
    persistence::{CommandWithCallback, DbCommand},
    structs::Context,
};

pub async fn is_guild_setup(
    query_handler: &UnboundedSender<DbCommand>,
    guild_id: u64,
) -> Result<bool, String> {
    let (callback, callback_recv) = oneshot::channel();

    let send_result = query_handler.send(DbCommand::CheckContainsGuild(CommandWithCallback {
        data: guild_id,
        callback,
    }));

    if send_result.is_err() {
        return Err("Cannot access data store".to_owned());
    }

    match callback_recv.await {
        Ok(Ok(val)) => Ok(val),
        Ok(Err(_)) => Err("Backing datastore returned error when fetching result".to_owned()),
        Err(e) => {
            println!("{}", e);
            Err("Could not contact data store when fetching result".to_owned())
        }
    }
}

pub async fn autocomplete_tz<'a, 'b>(
    _ctx: Context<'_>,
    partial: &'a str,
) -> impl Iterator<Item = AutocompleteChoice<String>> + 'a {
    TZ_NAMES
        .iter()
        .filter(|x| x.starts_with(partial.to_owned().as_str()))
        .take(5)
        .copied()
        .map(String::from)
        .map(AutocompleteChoice::from)
}

pub async fn get_disp_name_from_entry_name(
    entry_name: String,
    context: impl CacheHttp + Copy,
    guild_opt: Option<GuildId>,
) -> String {
    match parse_username(&entry_name) {
        Some(user_id) => {
            let user_id = UserId(user_id);
            match user_id.to_user(context).await {
                Ok(user) => match guild_opt {
                    Some(guild_id) => match user.nick_in(context, guild_id).await {
                        Some(nick) => nick,
                        None => user.name,
                    },
                    None => user.name,
                },
                Err(_) => "Unknown User".to_owned(),
            }
        }
        None => entry_name,
    }
}

pub fn calc_next_birthday(birthday: DateTime<Utc>) -> i64 {
    match birthday.timestamp() <= Utc::now().timestamp() {
        true => {
            let dur = chrono::Months::new(12 * (1 + Utc::now().years_since(birthday).unwrap_or(0)));
            match birthday.checked_add_months(dur) {
                Some(t) => t.timestamp(),
                None => approximate_next_birthday(birthday.timestamp()),
            }
        }
        false => birthday.timestamp(),
    }
}

pub fn approximate_next_birthday(timestamp: i64) -> i64 {
    let mut next_birthday = timestamp;
    let now = Utc::now().timestamp();

    while next_birthday <= now {
        match DateTime::<Utc>::from_timestamp(next_birthday, 0) {
            Some(nb) => {
                if nb.timestamp() % 4 == 0 {
                    next_birthday += 31_622_400
                } else {
                    next_birthday += 31_536_000
                }
            }
            None => next_birthday += 31_556_926,
        };
    }
    next_birthday
}
