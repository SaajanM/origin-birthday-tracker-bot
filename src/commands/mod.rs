mod bday;
mod ping;
mod setup;
mod shutdown;

use poise::Command;

use bday::*;
use ping::*;
use setup::*;
use shutdown::*;

use crate::structs::{Data, Error};

pub fn get_commands() -> Vec<Command<Data, Error>> {
    vec![setup(), ping(), shutdown(), bday()]
}
