CREATE TABLE entries (
    id          TEXT NOT NULL,
    created     INTEGER NOT NULL,
    version     INTEGER NOT NULL,
    current     BOOLEAN NOT NULL,
    title       TEXT NOT NULL,
    description TEXT NOT NULL,
    lat         FLOAT NOT NULL,
    lng         FLOAT NOT NULL,
    street      TEXT,
    zip         TEXT,
    city        TEXT,
    country     TEXT,
    email       TEXT,
    telephone   TEXT,
    homepage    TEXT,
    license     TEXT,
    PRIMARY KEY (id, version)
);

CREATE TABLE categories (
    id      TEXT PRIMARY KEY NOT NULL,
    created INTEGER NOT NULL,
    version INTEGER NOT NULL,
    name    TEXT    NOT NULL
);

CREATE TABLE entry_category_relations (
    entry_id      TEXT NOT NULL,
    entry_version INTEGER NOT NULL,
    category_id   TEXT NOT NULL,
    PRIMARY KEY (entry_id, entry_version, category_id),
    FOREIGN KEY (entry_id, entry_version) REFERENCES entries(id,version)
);

CREATE TABLE tags (
    id  TEXT PRIMARY KEY NOT NULL
);

CREATE TABLE comments (
  id      TEXT PRIMARY KEY NOT NULL,
  created INTEGER NOT NULL,
  text    TEXT NOT NULL
);

CREATE TABLE ratings (
    id      TEXT PRIMARY KEY NOT NULL,
    created INTEGER NOT NULL,
    title   TEXT NOT NULL,
    value   INTEGER NOT NULL,
    context TEXT NOT NULL,
    source  TEXT
);

CREATE TABLE bbox_subscriptions (
    id              TEXT PRIMARY KEY NOT NULL,
    south_west_lat  FLOAT NOT NULL,
    south_west_lng  FLOAT NOT NULL,
    north_east_lat  FLOAT NOT NULL,
    north_east_lng  FLOAT NOT NULL
);

CREATE TABLE triples (
    subject_id    TEXT NOT NULL,
    subject_type  TEXT NOT NULL,
    predicate     TEXT NOT NULL,
    object_id     TEXT NOT NULL,
    object_type   TEXT NOT NULL,
    PRIMARY KEY (subject_id, predicate, object_id)
);

CREATE TABLE users (
    username        TEXT PRIMARY KEY NOT NULL,
    password        TEXT    NOT NULL,
    email           TEXT    NOT NULL,
    email_confirmed BOOLEAN NOT NULL
);