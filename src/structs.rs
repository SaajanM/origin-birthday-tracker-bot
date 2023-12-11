use std::sync::{atomic::AtomicBool, Arc};
use tokio::sync::mpsc::UnboundedSender;

use crate::persistence::DbCommand;

pub struct Data {
    pub query_handler: UnboundedSender<DbCommand>,
    pub exit_flag: Arc<AtomicBool>,
} // User data, which is stored and accessible in all command invocations'

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
