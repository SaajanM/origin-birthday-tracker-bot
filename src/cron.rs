use std::{sync::Arc, time::Duration};

use chrono::{Datelike, Utc};
use poise::serenity_prelude::{ChannelId, Mention, UserId};
use serenity::CacheAndHttp;

use crate::origin_bot::{BirthdayInfo, Data};

pub async fn bday_crunching(context: Arc<CacheAndHttp>, data: Data) {
    let mut interval_timer = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval_timer.tick().await;

        let global_reader = data.state.guild_map.read().await;
        println!("Global Reader obtained");
        for (_, guild_data) in global_reader.iter() {
            let mut continue_checking = true;
            while continue_checking {
                let channel = {
                    let guild_reader = guild_data.read().await;
                    println!("Guild Reader obtained");
                    let bday_check = guild_reader.schedule.first();

                    let bday_check = match bday_check {
                        Some(bday_check) => bday_check,

                        // No registered users in guild
                        None => {
                            continue_checking = false;
                            continue;
                        }
                    };

                    if bday_check.datetime > Utc::now() {
                        continue_checking = false;
                        continue;
                    }
                    let user_id = bday_check.associated_user;

                    // Mention user in the requisite guild
                    let channel_id = match guild_reader.announcement_channel {
                        Some(id) => id,
                        None => {
                            continue_checking = false;
                            continue;
                        }
                    };
                    let channel = ChannelId(channel_id);
                    let user = UserId(user_id);

                    let _ = channel
                        .say(
                            &context.http,
                            format!("Happy Birthday {} :tada::tada::tada:", Mention::User(user)),
                        )
                        .await;
                    channel
                };
                // Reinsert into both
                let mut guild_writer = guild_data.write().await;
                println!("Guild Writer obtained");

                if let Some(removed_info) = guild_writer.schedule.pop_first() {
                    if guild_writer
                        .birthday_map
                        .remove(&removed_info.associated_user)
                        .is_some()
                    {
                        // Make new info
                        let new_datetime = removed_info
                            .datetime
                            .with_year(removed_info.datetime.year() + 1);

                        match new_datetime {
                            Some(new_datetime) => {
                                let new_info = Arc::new(BirthdayInfo {
                                    datetime: new_datetime,
                                    associated_user: removed_info.associated_user,
                                });

                                guild_writer
                                    .birthday_map
                                    .insert(removed_info.associated_user, Arc::clone(&new_info));
                                guild_writer.schedule.insert(new_info);
                            }
                            None => {
                                let _ = channel
                                            .say(
                                                &context.http,
                                                "Could not add back the birthday, please manually reinsert its",
                                            )
                                            .await;
                            }
                        }
                    }
                }
            }
        }
    }
}
