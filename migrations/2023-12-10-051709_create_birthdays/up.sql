CREATE TABLE birthdays (
    id INTEGER NOT NULL PRIMARY KEY,
    birthday BIGINT NOT NULL,
    who_to_ping BIGINT NULL,
    entry_name TEXT NULL,
    guild_id BIGINT NOT NULL REFERENCES guilds(guild_id)
)