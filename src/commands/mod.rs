mod ping;
mod setup;

use ping::*;
use poise::Command;
use setup::*;

use crate::structs::{Data, Error};

pub fn get_commands() -> Vec<Command<Data, Error>> {
    vec![setup(), ping()]
}
