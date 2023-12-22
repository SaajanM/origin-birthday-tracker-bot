use diesel::prelude::*;

#[derive(Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::guilds)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Guild {
    pub id: i32,
    pub guild_id: i64,
    pub announcement_channel: i64,
    pub allows_anyone_edit: bool,
    pub do_server_birthday: bool,
    pub timezone_name: Option<String>,
}

#[derive(Insertable, PartialEq, Debug)]
#[diesel(table_name = crate::schema::guilds)]
pub struct NewGuild {
    pub guild_id: i64,
    pub announcement_channel: i64,
    pub allows_anyone_edit: bool,
    pub do_server_birthday: bool,
    pub timezone_name: Option<String>,
}

#[derive(Queryable, Selectable, Identifiable, AsChangeset)]
#[diesel(table_name = crate::schema::birthdays)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct Birthday {
    pub id: i32,
    pub birthday: i64,
    pub next_birthday: i64,
    pub uses_time: bool,
    pub who_to_ping: i64,
    pub entry_name: String,
    pub guild_id: i64,
}

#[derive(Insertable, PartialEq, Debug)]
#[diesel(table_name = crate::schema::birthdays)]
pub struct NewBirthday {
    pub birthday: i64,
    pub next_birthday: i64,
    pub uses_time: bool,
    pub who_to_ping: i64,
    pub entry_name: String,
    pub guild_id: i64,
}
