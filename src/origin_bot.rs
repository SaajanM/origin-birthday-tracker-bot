use std::{path::PathBuf, sync::Arc};

use crate::{
    commands::get_commands,
    cron::bday_crunching,
    persistence::SaveManager,
    structs::{ApplicationState, Data},
};
use serenity::prelude::GatewayIntents;
use tokio::fs;

pub async fn start_bot(
    token: String,
    intents: GatewayIntents,
    save_location: PathBuf,
) -> Result<(), serenity::Error> {
    let try_load = fs::read_to_string(save_location.clone()).await;

    let state = if let Ok(loaded_data) = try_load {
        if let Ok(state) = serde_json::from_str::<ApplicationState>(&loaded_data) {
            state
        } else {
            Default::default()
        }
    } else {
        Default::default()
    };

    let application_state = Arc::new(state);

    let saver = Arc::new(SaveManager::new(
        Arc::clone(&application_state),
        save_location,
    ));

    let cron_data = Data {
        state: Arc::clone(&application_state),
        saver: Arc::clone(&saver),
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
                    saver,
                })
            })
        });

    let framework = framework_builder.build().await?;

    let http_cache = Arc::clone(&framework.client().cache_and_http);

    tokio::spawn(bday_crunching(http_cache, cron_data));

    framework.start().await
}
