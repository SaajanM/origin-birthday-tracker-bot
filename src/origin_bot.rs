use std::{
    collections::{BTreeSet, HashMap},
    sync::Arc,
};

use crate::{commands::get_commands, cron::bday_crunching};
use chrono::{DateTime, Utc};
use chrono_tz::Tz;
use serenity::prelude::GatewayIntents;
use tokio::sync::RwLock;

#[derive(Default)]
pub struct Data {
    pub state: Arc<ApplicationState>,
} // User data, which is stored and accessible in all command invocations'

#[derive(Default)]
pub struct ApplicationState {
    pub guild_map: RwLock<HashMap<u64, RwLock<GuildData>>>,
}

#[derive(Default)]
pub struct GuildData {
    pub timezone: Option<Tz>,
    pub announcement_channel: Option<u64>,
    pub schedule: BTreeSet<Arc<BirthdayInfo>>,
    // Exists purely for fast deletion
    pub birthday_map: HashMap<u64, Arc<BirthdayInfo>>,
}

#[derive(PartialEq, PartialOrd, Eq, Ord)]
pub struct BirthdayInfo {
    pub datetime: DateTime<Utc>,
    pub associated_user: u64,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;

pub async fn start_bot(token: String, intents: GatewayIntents) -> Result<(), serenity::Error> {
    let application_state = Arc::new(ApplicationState::default());

    let cron_data = Data {
        state: Arc::clone(&application_state),
    };

    let framework_builder = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: get_commands(),
            ..Default::default()
        })
        .token(token)
        .intents(intents)
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;
                Ok(Data {
                    state: application_state,
                })
            })
        });

    let framework = framework_builder.build().await?;

    let http_cache = Arc::clone(&framework.client().cache_and_http);

    tokio::spawn(bday_crunching(http_cache, cron_data));

    framework.start().await
}
