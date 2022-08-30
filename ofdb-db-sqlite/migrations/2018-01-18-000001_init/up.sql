CREATE TABLE entries (
    id          TEXT NOT NULL,
    osm_node    INTEGER,
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
    FOREIGN KEY (entry_id, entry_version) REFERENCES entries(id,version),
    FOREIGN KEY (category_id) REFERENCES categories(id)
);

CREATE TABLE entry_tag_relations (
    entry_id      TEXT NOT NULL,
    entry_version INTEGER NOT NULL,
    tag_id        TEXT NOT NULL,
    PRIMARY KEY (entry_id, entry_version, tag_id),
    FOREIGN KEY (entry_id, entry_version) REFERENCES entries(id,version),
    FOREIGN KEY (tag_id) REFERENCES tags(id)
);

CREATE TABLE tags (
    id  TEXT PRIMARY KEY NOT NULL
);

CREATE TABLE comments (
  id        TEXT PRIMARY KEY NOT NULL,
  created   INTEGER NOT NULL,
  text      TEXT NOT NULL,
  rating_id TEXT NOT NULL,
  FOREIGN KEY (rating_id) REFERENCES ratings(id)
);

CREATE TABLE ratings (
    id      TEXT PRIMARY KEY NOT NULL,
    created INTEGER NOT NULL,
    title   TEXT NOT NULL,
    value   INTEGER NOT NULL,
    context TEXT NOT NULL,
    source  TEXT,
    entry_id TEXT NOT NULL
);

CREATE TABLE bbox_subscriptions (
    id              TEXT PRIMARY KEY NOT NULL,
    south_west_lat  FLOAT NOT NULL,
    south_west_lng  FLOAT NOT NULL,
    north_east_lat  FLOAT NOT NULL,
    north_east_lng  FLOAT NOT NULL,
    username        TEXT  NOT NULL,
    FOREIGN KEY (username) REFERENCES users(username)
);

CREATE TABLE users (
    id              TEXT NOT NULL, -- TODO: remove
    username        TEXT PRIMARY KEY NOT NULL,
    password        TEXT    NOT NULL,
    email           TEXT    NOT NULL,
    email_confirmed BOOLEAN NOT NULL
);
