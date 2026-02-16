PRAGMA synchronous = NORMAL;
PRAGMA journal_mode = WAL;

CREATE TABLE users (
    pub_key BLOB PRIMARY KEY,
    name TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    address TEXT,
    signature BLOB NOT NULL,
    trust INTEGER NOT NULL,
) STRICT;

CREATE TABLE novels (
    hash TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    release_date INTEGER NOT NULL,
    source BLOB NOT NULL,
    signature BLOB NOT NULL,
) STRICT;

CREATE TABLE novel_chapters (
    signature BLOB PRIMARY KEY,
    source BLOB NOT NULL,
    index_hash BLOB NOT NULL,
    timestamp INTEGER NOT NULL,
    magnet_link TEXT NOT NULL,
    entries

) STRICT;
