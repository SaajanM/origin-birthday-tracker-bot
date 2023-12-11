// @generated automatically by Diesel CLI.

diesel::table! {
    birthdays (id) {
        id -> Integer,
        birthday -> BigInt,
        who_to_ping -> Nullable<BigInt>,
        entry_name -> Nullable<Text>,
        guild_id -> BigInt,
    }
}

diesel::table! {
    guilds (id) {
        id -> Integer,
        guild_id -> BigInt,
        announcement_channel -> BigInt,
        allows_anyone_edit -> Bool,
        do_server_birthday -> Bool,
        timezone_name -> Nullable<Text>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(birthdays, guilds,);
