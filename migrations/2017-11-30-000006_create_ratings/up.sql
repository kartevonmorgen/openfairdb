CREATE TABLE ratings (
    id      TEXT PRIMARY KEY NOT NULL,
    created INTEGER NOT NULL,
    title   TEXT NOT NULL,
    value   INTEGER NOT NULL,
    context TEXT NOT NULL,
    source  TEXT
);
