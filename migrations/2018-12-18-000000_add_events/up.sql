CREATE TABLE events (
    id          TEXT PRIMARY KEY NOT NULL,
    title       TEXT NOT NULL,
    description TEXT,
    start       INTEGER NOT NULL,
    end         INTEGER,
    lat         FLOAT,
    lng         FLOAT,
    street      TEXT,
    zip         TEXT,
    city        TEXT,
    country     TEXT,
    email       TEXT,
    telephone   TEXT,
    homepage    TEXT,
    created_by  TEXT,
    FOREIGN KEY (created_by) REFERENCES users(username)
);

CREATE TABLE event_tag_relations (
    event_id TEXT NOT NULL,
    tag_id   TEXT NOT NULL,
    PRIMARY KEY (event_id, tag_id),
    FOREIGN KEY (event_id) REFERENCES events(id),
    FOREIGN KEY (tag_id) REFERENCES tags(id)
);
