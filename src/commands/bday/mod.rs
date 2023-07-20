use self::del::del;
use self::get::get;
use self::list::list;
use self::set::set;
use self::today::today;
use crate::structs::{Context, Error};

mod del;
mod get;
mod list;
mod set;
mod today;

/// Parent Command for all birthdat relayed doodads
#[poise::command(slash_command, subcommands("set", "del", "list", "get", "today"))]
pub async fn bday(ctx: Context<'_>) -> Result<(), Error> {
    ctx.say("Use one of the subcommands to alter this guilds birthday list")
        .await?;
    Ok(())
}
