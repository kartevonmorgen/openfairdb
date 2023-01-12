CREATE TABLE review_tokens (
    rowid         INTEGER PRIMARY KEY NOT NULL,
    --
    place_rowid   INTEGER NOT NULL,
    --
    revision      INTEGER, -- revision number or NULL if the place is new and has not been cleared yet
    expires_at    INTEGER NOT NULL,
    nonce         TEXT    NOT NULL,
    --
    UNIQUE        (nonce),
    FOREIGN KEY   (place_rowid) REFERENCES place(rowid)
);
