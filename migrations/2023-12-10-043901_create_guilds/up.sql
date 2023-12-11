CREATE TABLE guilds (
    id INTEGER NOT NULL PRIMARY KEY,
    guild_id BIGINT UNIQUE NOT NULL,
    announcement_channel BIGINT UNIQUE NOT NULL,
    allows_anyone_edit BOOLEAN NOT NULL,
    do_server_birthday BOOLEAN NOT NULL,
    timezone_name TEXT NULL
)