mod add;
mod list;
mod remove;

use crate::structs::{Context, Error};
use add::*;
use list::*;
use remove::*;

/// Parent Command for all birthday relayed commands
#[poise::command(slash_command, subcommands("list", "add", "remove"))]
pub async fn bday(_ctx: Context<'_>) -> Result<(), Error> {
    Ok(())
}
