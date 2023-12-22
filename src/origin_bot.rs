use std::sync::{atomic::AtomicBool, Arc};

use crate::{
    commands::get_commands, cron::bday_crunching, persistence::SaveManager, structs::Data,
};
use poise::serenity_prelude::Activity;
use serenity::prelude::GatewayIntents;

pub async fn start_bot(
    token: String,
    intents: GatewayIntents,
    save_location: String,
) -> Result<(), anyhow::Error> {
    // Generate 3 tasks
    // Task 1: deal with real time discord interaction (commands, etc)
    // Task 2: deal with accessing the database
    // Task 3: deal with async discord integration
    // Task 2 must happen first to return a handle the others can use to make queries
    // Task 3 must launch next to provide a handle to trigger a queuing of a cron job
    // Task 1 comes last, though the framework must be instantiated here before any of these tasks
    let global_exit_flag = Arc::new(AtomicBool::new(false));

    let save_manager_exit_flag = Arc::clone(&global_exit_flag);
    let save_manager = SaveManager::try_new(save_location, save_manager_exit_flag)?;

    let framework_exit_flag = Arc::clone(&global_exit_flag);
    let framework_query_handler = save_manager.db_query_channel.clone();
    let framework_builder = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: get_commands(),
            ..Default::default()
        })
        .token(token)
        .intents(intents)
        .setup(|ctx, _ready, framework| {
            Box::pin(async move {
                ctx.set_presence(
                    Some(Activity::watching("for celebrations")),
                    poise::serenity_prelude::OnlineStatus::Online,
                )
                .await;
                poise::builtins::register_globally(ctx, &framework.options().commands).await?;

                Ok(Data {
                    query_handler: framework_query_handler,
                    exit_flag: framework_exit_flag,
                    shard_manager: Arc::clone(framework.shard_manager()),
                })
            })
        });

    let framework = framework_builder.build().await?;

    let http_cache = Arc::clone(&framework.client().cache_and_http);
    let cron_exit_flag = Arc::clone(&global_exit_flag);
    let cron_query_handler = save_manager.db_query_channel.clone();
    let cron_data = Data {
        exit_flag: cron_exit_flag,
        query_handler: cron_query_handler,
        shard_manager: Arc::clone(framework.shard_manager()),
    };

    tokio::spawn(bday_crunching(http_cache, cron_data));
    println!("Starting Origin Bot...");

    framework.start().await?;
    Ok(())
}
