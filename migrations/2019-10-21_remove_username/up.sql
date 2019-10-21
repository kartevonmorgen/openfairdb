CREATE TABLE users_new (
    id              INTEGER PRIMARY KEY,
    email           TEXT    NOT NULL COLLATE NOCASE,
    email_confirmed BOOLEAN,
    password        TEXT    NOT NULL,
    role            INTEGER NOT NULL,
    UNIQUE(email)
);

INSERT INTO users_new
SELECT rowid AS id, LOWER(email), MAX(email_confirmed) AS email_confirmed, password, MAX(role) AS role
FROM users
GROUP BY LOWER(email)
HAVING role = MAX(role);

CREATE TEMPORARY TABLE username_id (
    user_id INTEGER NOT NULL,
    username TEXT NOT NULL,
    UNIQUE(username)
);

INSERT INTO username_id
SELECT users_new.id, users.username
FROM users_new INNER JOIN users
ON users_new.email = LOWER(users.email);

CREATE TABLE events_new (
    id             INTEGER PRIMARY KEY,
    uid            TEXT NOT NULL,
    title          TEXT NOT NULL,
    description    TEXT,
    start          INTEGER NOT NULL,
    end            INTEGER,
    lat            FLOAT,
    lng            FLOAT,
    street         TEXT,
    zip            TEXT,
    city           TEXT,
    country        TEXT,
    email          TEXT,
    telephone      TEXT,
    homepage       TEXT,
    created_by     INTEGER,
    registration   INTEGER,
    organizer      TEXT,
    image_url      TEXT,
    image_link_url TEXT,
    archived       INTEGER DEFAULT NULL,
    FOREIGN KEY (created_by) REFERENCES users_new(id)
);

INSERT INTO events_new
SELECT events.rowid AS id, events.id AS uid, title, description, start, end, lat, lng, street, zip, city, country, events.email, telephone, homepage, user_id, registration, organizer, image_url, image_link_url, archived
FROM events
LEFT OUTER JOIN username_id
ON LOWER(created_by) = username;

CREATE TABLE event_tags (
    event_id INTEGER NOT NULL,
    tag      TEXT NOT NULL,
    PRIMARY KEY (event_id, tag),
    FOREIGN KEY (event_id) REFERENCES events_new(id)
);

CREATE INDEX event_tag_index ON event_tags (tag);

INSERT INTO event_tags
SELECT events_new.id AS event_id, tag_id as tag
FROM event_tag_relations JOIN events_new ON event_id=events_new.uid;

CREATE TABLE bbox_subscriptions_new (
    id              INTEGER PRIMARY KEY NOT NULL,
    uid             TEXT NOT NULL,
    user_id         INTEGER NOT NULL,
    south_west_lat  FLOAT NOT NULL,
    south_west_lng  FLOAT NOT NULL,
    north_east_lat  FLOAT NOT NULL,
    north_east_lng  FLOAT NOT NULL,
    UNIQUE(uid),
    FOREIGN KEY (user_id) REFERENCES users_new(id)
);

INSERT INTO bbox_subscriptions_new
SELECT bbox_subscriptions.rowid AS id, bbox_subscriptions.id AS uid, user_id, south_west_lat, south_west_lng, north_east_lat, north_east_lng
FROM bbox_subscriptions JOIN username_id ON LOWER(bbox_subscriptions.username)=username_id.username;

CREATE TABLE user_tokens (
    id          INTEGER PRIMARY KEY,
    user_id     INTEGER NOT NULL,
    expires_at  INTEGER NOT NULL,
    nonce       TEXT NOT NULL,
    UNIQUE      (user_id),
    UNIQUE      (nonce),
    FOREIGN KEY (user_id) REFERENCES users_new(id)
);

INSERT INTO user_tokens
SELECT email_token_credentials.id, user_id, expires_at, nonce
FROM email_token_credentials JOIN username_id ON LOWER(email_token_credentials.username)=username_id.username;

DROP TABLE temp.username_id;

DROP TABLE event_tag_relations;

DROP TABLE events;
ALTER TABLE events_new RENAME TO events;

DROP INDEX bbox_subscriptions_fk_username;

DROP TABLE bbox_subscriptions;
ALTER TABLE bbox_subscriptions_new RENAME TO bbox_subscriptions;

DROP TABLE email_token_credentials;

DROP TABLE users;
ALTER TABLE users_new RENAME TO users;

-- Recreate indexes
CREATE INDEX events_fk_created_by ON events (created_by);
