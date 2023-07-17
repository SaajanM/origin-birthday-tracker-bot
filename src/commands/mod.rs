mod bday;
mod set_channel;
mod timezone;

use bday::*;
use poise::Command;
use set_channel::*;
use timezone::*;

use crate::origin_bot::{Data, Error};

pub fn get_commands() -> Vec<Command<Data, Error>> {
    vec![bday(), timezone(), channel()]
}
