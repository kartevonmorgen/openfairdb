CREATE TABLE sent_reminders (
    rowid           INTEGER PRIMARY KEY NOT NULL,
    --
    place_rowid     INTEGER NOT NULL,
    --
    sent_at         INTEGER NOT NULL,
    sent_to_email   TEXT NOT NULL,
    --
    FOREIGN KEY (place_rowid) REFERENCES place(rowid)
);
