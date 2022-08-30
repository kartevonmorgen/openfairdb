CREATE TABLE place_revision_custom_link (
    -- implicit/anonymous integer primary key, i.e. hidden rowid column
    parent_rowid INTEGER NOT NULL,
    --
    url          TEXT NOT NULL,
    title        TEXT,
    description  TEXT,
    PRIMARY KEY (parent_rowid, url),
    FOREIGN KEY (parent_rowid) REFERENCES place_revision(rowid)
);
