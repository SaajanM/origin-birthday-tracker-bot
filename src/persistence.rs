use anyhow::anyhow;
use diesel::sqlite::Sqlite;
use diesel::{dsl::count_distinct, prelude::*};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::sync::mpsc::{unbounded_channel, UnboundedReceiver, UnboundedSender};

use crate::models::{Birthday, Guild, NewBirthday};
use crate::{models::NewGuild, schema};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("./migrations");

pub struct CommandWithCallback<CommandDataT, CallbackDataT> {
    pub data: CommandDataT,
    pub callback: tokio::sync::oneshot::Sender<anyhow::Result<CallbackDataT>>,
}

pub enum DbCommand {
    AddGuild(CommandWithCallback<NewGuild, Guild>),
    CheckContainsGuild(CommandWithCallback<u64, bool>),
    Shutdown,
    FetchNBirthdays(CommandWithCallback<(u64, i32), (Vec<Birthday>, u64)>),
    GetGuildData(CommandWithCallback<u64, Guild>),
    AddBirthday(CommandWithCallback<NewBirthday, Birthday>),
    RemoveBirthday(CommandWithCallback<(u64, String), ()>),
    GetBirthdaysBetween(CommandWithCallback<(i64, i64), Vec<(Birthday, Guild)>>),
    SetNextBirthday(CommandWithCallback<(i32, i64), ()>),
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
                DbCommand::Shutdown => exit_flag.store(true, Ordering::Relaxed),
                DbCommand::FetchNBirthdays(command) => fetch_n_birthdays(command, &mut db),
                DbCommand::GetGuildData(command) => get_guild_data(command, &mut db),
                DbCommand::AddBirthday(command) => add_birthday(command, &mut db),
                DbCommand::RemoveBirthday(command) => remove_birthday(command, &mut db),
                DbCommand::GetBirthdaysBetween(command) => birthdays_between(command, &mut db),
                DbCommand::SetNextBirthday(command) => set_next_birthday(command, &mut db),
            },
            _ => break,
        }
    }
    exit_flag.swap(true, Ordering::Relaxed);
}

fn add_guild(command: CommandWithCallback<NewGuild, Guild>, db: &mut SqliteConnection) {
    let insert = diesel::insert_into(schema::guilds::table)
        .values(command.data)
        .returning(Guild::as_returning())
        .get_result(db);
    let _ = command.callback.send(insert.map_err(anyhow::Error::from));
}

fn check_contains_guild(command: CommandWithCallback<u64, bool>, db: &mut SqliteConnection) {
    let gid = command.data as i64;
    let count_result = schema::guilds::table
        .select(count_distinct(schema::guilds::guild_id.eq(gid)))
        .first::<i64>(db);
    let result = count_result.map(|val| val > 0).map_err(anyhow::Error::from);

    let _ = command.callback.send(result);
}

fn fetch_n_birthdays(
    command: CommandWithCallback<(u64, i32), (Vec<Birthday>, u64)>,
    db: &mut SqliteConnection,
) {
    let gid = command.data.0 as i64;
    let limit = command.data.1 as i64;

    let fetch_result: QueryResult<Vec<Birthday>> = schema::birthdays::table
        .limit(limit)
        .order(schema::birthdays::next_birthday.asc())
        .filter(schema::birthdays::guild_id.eq(gid))
        .load::<Birthday>(db);

    let count_result: QueryResult<i64> = schema::birthdays::table
        .filter(schema::birthdays::guild_id.eq(gid))
        .select(count_distinct(schema::birthdays::id))
        .first::<i64>(db);

    let _ = count_result.as_ref().map(|x| {
        println!("{}", x);
        x
    });

    let count_result = count_result.map(|val| (if val <= limit { 0 } else { val - limit }) as u64);

    let result = match (fetch_result, count_result) {
        (Ok(v), Ok(i)) => {
            println!("{}", i);
            Ok((v, i))
        }
        (Ok(_), Err(e)) => Err(anyhow::Error::from(e)),
        (Err(e), Ok(_)) => Err(anyhow::Error::from(e)),
        (Err(e1), Err(e2)) => Err(anyhow::anyhow!("{};{}", e1, e2)),
    };

    let _ = command.callback.send(result);
}

fn get_guild_data(command: CommandWithCallback<u64, Guild>, db: &mut SqliteConnection) {
    let gid = command.data as i64;

    let result: QueryResult<Guild> = schema::guilds::table
        .filter(schema::guilds::guild_id.eq(gid))
        .first::<Guild>(db);

    let _ = command.callback.send(result.map_err(anyhow::Error::from));
}

fn add_birthday(command: CommandWithCallback<NewBirthday, Birthday>, db: &mut SqliteConnection) {
    let insert = diesel::insert_into(schema::birthdays::table)
        .values(command.data)
        .returning(crate::models::Birthday::as_returning())
        .get_result(db);
    let _ = command.callback.send(insert.map_err(anyhow::Error::from));
}

fn remove_birthday(command: CommandWithCallback<(u64, String), ()>, db: &mut SqliteConnection) {
    // let delete = diesel::insert_into(schema::birthdays::table)
    //     .values(command.data)
    //     .returning(crate::models::Birthday::as_returning())
    //     .get_result(db);
    let gid = command.data.0 as i64;
    let e_name = command.data.1;
    let delete = diesel::delete(
        schema::birthdays::table.filter(
            schema::birthdays::entry_name
                .eq(e_name)
                .and(schema::birthdays::guild_id.eq(gid)),
        ),
    )
    .execute(db);
    let _ = command
        .callback
        .send(delete.map(|_| ()).map_err(anyhow::Error::from));
}

fn birthdays_between(
    command: CommandWithCallback<(i64, i64), Vec<(Birthday, Guild)>>,
    db: &mut SqliteConnection,
) {
    let min_ts = command.data.0;
    let max_ts = command.data.1;

    let bdays: QueryResult<Vec<(Birthday, Guild)>> = schema::birthdays::table
        .filter(schema::birthdays::next_birthday.between(min_ts, max_ts))
        .inner_join(
            schema::guilds::table.on(schema::birthdays::guild_id.eq(schema::guilds::guild_id)),
        )
        .select((Birthday::as_select(), Guild::as_select()))
        .get_results::<(Birthday, Guild)>(db);

    let _ = command.callback.send(bdays.map_err(anyhow::Error::from));
}

fn set_next_birthday(command: CommandWithCallback<(i32, i64), ()>, db: &mut SqliteConnection) {
    let row_id = command.data.0;
    let new_next = command.data.1;

    let update_result = diesel::update(schema::birthdays::table)
        .filter(schema::birthdays::id.eq(row_id))
        .set(schema::birthdays::next_birthday.eq(new_next))
        .execute(db);

    let _ = command
        .callback
        .send(update_result.map(|_| ()).map_err(anyhow::Error::from));
}
