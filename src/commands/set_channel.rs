use crate::origin_bot::{Context, Error};
use poise::serenity_prelude::Channel;

/// Set the channel where messages will appear (MUST BE RUN)
#[poise::command(slash_command)]
pub async fn channel(
    ctx: Context<'_>,
    #[description = "The channel that messages will be sent in"] channel: Channel,
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

    let mut guild_entry_write = guild_entry.write().await;

    guild_entry_write.announcement_channel = Some(channel.id().0);

    ctx.say("Channel successfully set!").await?;

    Ok(())
}
