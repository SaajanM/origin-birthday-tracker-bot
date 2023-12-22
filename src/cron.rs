use chrono::{DateTime, Utc};
use poise::serenity_prelude::ChannelId;
use serenity::{utils::parse_username, CacheAndHttp};
use std::sync::{atomic::Ordering, Arc};
use tokio::sync::{mpsc::UnboundedSender, oneshot};

use crate::{
    helpers::{approximate_next_birthday, calc_next_birthday},
    persistence::{CommandWithCallback, DbCommand},
    structs::Data,
};

pub async fn bday_crunching(context: Arc<CacheAndHttp>, data: Data) {
    let std_duration = std::time::Duration::from_secs(60);
    let mut interval_timer = tokio::time::interval(std_duration);
    let query_handler = data.query_handler;

    let mut last_timestamp = Utc::now().timestamp();

    while !data.exit_flag.load(Ordering::Relaxed) {
        let (callback, callback_recv) = oneshot::channel();
        let now = Utc::now().timestamp();
        let query_result =
            query_handler.send(DbCommand::GetBirthdaysBetween(CommandWithCallback {
                data: (last_timestamp, now),
                callback,
            }));
        last_timestamp = now;

        if query_result.is_err() {
            println!("Could not send out messages!!!")
        }

        match callback_recv.await {
            Ok(Ok(bdays)) => {
                for (bday, guild) in bdays {
                    say_birthday_message(&bday, &guild, &context).await;
                    update_next_bday(&bday, &guild, &query_handler).await;
                }
            }
            _ => println!("Could not send out all messages!!!"),
        }

        interval_timer.tick().await;
    }
    data.shard_manager.lock().await.shutdown_all().await
}

async fn update_next_bday(
    bday: &crate::models::Birthday,
    _guild: &crate::models::Guild,
    query_handler: &UnboundedSender<DbCommand>,
) {
    let bday_instant = DateTime::<Utc>::from_timestamp(bday.next_birthday, 0);
    let new_timestamp = match bday_instant {
        Some(bday_instant) => calc_next_birthday(bday_instant),
        None => approximate_next_birthday(bday.next_birthday),
    };
    let (callback, callback_recv) = oneshot::channel();

    let query_result = query_handler.send(DbCommand::SetNextBirthday(CommandWithCallback {
        data: (bday.id, new_timestamp),
        callback,
    }));

    if query_result.is_err() {
        println!("Could not update next birthday!!!");
    }

    match callback_recv.await {
        Ok(Ok(_)) => {}
        _ => {
            println!("Could not update next birthday!!!");
        }
    }
}

async fn say_birthday_message(
    bday: &crate::models::Birthday,
    guild: &crate::models::Guild,
    context: &Arc<CacheAndHttp>,
) {
    let channel_id = ChannelId(guild.announcement_channel as u64);

    let mut message = format!("Happy birthday {}", bday.entry_name);

    match parse_username(bday.entry_name.clone()) {
        Some(_) => {}
        None => message += format!("  ||<@{}>||", bday.who_to_ping as u64).as_str(),
    }

    message += "!!!";

    match channel_id.say(&context.http, message).await {
        Ok(_) => {}
        Err(_) => println!("A message was not sent!!!"),
    }
}
