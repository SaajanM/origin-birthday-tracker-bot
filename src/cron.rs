use std::{sync::Arc, time::Duration};

use chronoutil::delta;
use poise::serenity_prelude::{ChannelId, Mention, UserId};
use serenity::CacheAndHttp;

use crate::structs::{BirthdayInfo, Data};

pub async fn bday_crunching(context: Arc<CacheAndHttp>, data: Data) {
    let mut interval_timer = tokio::time::interval(Duration::from_secs(900));
    loop {
        interval_timer.tick().await;

        let global_reader = data.state.guild_map.read().await;
        println!("Global Reader obtained");
        for (guild_id, guild_data) in global_reader.iter() {
            let happened_bdays = {
                let mut writer = guild_data.rw_lock.write().await;
                writer.birthday_schedule.pop_occured()
            };
            let announcement_channel = { guild_data.rw_lock.read().await.announcement_channel };
            for bday in happened_bdays {
                let user_id = bday.associated_user;
                let channel_id = announcement_channel.unwrap_or_default();
                let channel = ChannelId(channel_id);
                let user = UserId(user_id);

                if channel
                    .say(
                        &context.http,
                        format!("Happy Birthday {} :tada::tada::tada:", Mention::User(user)),
                    )
                    .await
                    .is_err()
                {
                    println!(
                        "Could not wish happy birthday to {} on channel {} on server {}",
                        user_id, channel_id, guild_id
                    )
                };

                let new_insert = Arc::new(BirthdayInfo {
                    datetime: delta::shift_years(bday.datetime, 1),
                    associated_user: user_id,
                });

                let mut writer = guild_data.rw_lock.write().await;
                let _ = writer.birthday_schedule.insert(new_insert);
            }
            data.saver.save();
        }
    }
}
