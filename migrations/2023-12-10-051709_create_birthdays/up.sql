CREATE TABLE birthdays (
    id INTEGER NOT NULL PRIMARY KEY,
    next_birthday BIGINT NOT NULL,
    uses_time BOOLEAN NOT NULL,
    who_to_ping BIGINT NOT NULL,
    entry_name TEXT NOT NULL,
    guild_id BIGINT NOT NULL REFERENCES guilds(guild_id),
    UNIQUE(entry_name, guild_id) ON CONFLICT ROLLBACK
)