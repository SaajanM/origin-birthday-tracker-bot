use anyhow::anyhow;
use diesel::sqlite::Sqlite;
use diesel::{dsl::count_distinct, prelude::*};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::schema::guilds::guild_id;
use crate::{models::NewGuild, schema};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub struct CommandWithCallback<CommandDataT, CallbackDataT> {
    pub data: CommandDataT,
    pub callback: tokio::sync::oneshot::Sender<CallbackDataT>,
}

pub enum DbCommand {
    AddGuild(CommandWithCallback<NewGuild, anyhow::Result<crate::models::Guild>>),
    CheckContainsGuild(CommandWithCallback<u64, anyhow::Result<bool>>),
}

pub struct SaveManager {
    pub db_query_channel: UnboundedSender<DbCommand>,
}

impl SaveManager {
    pub fn try_new(db_path: String, exit_flag: Arc<AtomicBool>) -> anyhow::Result<Self> {
        let mut db = SqliteConnection::establish(&db_path)?;

        Self::run_migrations(&mut db).map_err(|e| anyhow!(e))?;

        let (sender, receiver) = unbounded_channel::<DbCommand>();

        tokio::spawn(async move { handle_db_queries(exit_flag, receiver, db).await });

        Ok(SaveManager {
            db_query_channel: sender,
        })
    }

    fn run_migrations(
        connection: &mut impl MigrationHarness<Sqlite>,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync + 'static>> {
        // This will run the necessary migrations.
        //
        // See the documentation for `MigrationHarness` for
        // all available methods.
        connection.run_pending_migrations(MIGRATIONS)?;

        Ok(())
    }
}

async fn handle_db_queries(
    exit_flag: Arc<AtomicBool>,
    mut query_receiver: UnboundedReceiver<DbCommand>,
    mut db: SqliteConnection,
) {
    println!("Connected to DB, now processing db commands");
    while !exit_flag.load(Ordering::Relaxed) {
        match query_receiver.recv().await {
            Some(query) => match query {
                DbCommand::AddGuild(command) => add_guild(command, &mut db),
                DbCommand::CheckContainsGuild(command) => check_contains_guild(command, &mut db),
            },
            _ => break,
        }
    }
    exit_flag.swap(true, Ordering::Relaxed);
}

fn add_guild(
    command: CommandWithCallback<NewGuild, anyhow::Result<crate::models::Guild>>,
    db: &mut SqliteConnection,
) {
    let insert = diesel::insert_into(schema::guilds::table)
        .values(command.data)
        .returning(crate::models::Guild::as_returning())
        .get_result(db);
    let _ = command.callback.send(insert.map_err(anyhow::Error::from));
}

fn check_contains_guild(
    command: CommandWithCallback<u64, anyhow::Result<bool>>,
    db: &mut SqliteConnection,
) {
    let gid = command.data as i64;
    let count_result = schema::guilds::table
        .select(count_distinct(guild_id.eq(gid)))
        .first::<i64>(db);
    let result = count_result.map(|val| val > 0).map_err(anyhow::Error::from);

    let _ = command.callback.send(result);
}
