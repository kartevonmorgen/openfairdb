-- Disable fk constraints temporarily to allow inserting NULL references
-- into new tables with foreign keys.
PRAGMA foreign_keys = OFF;

-- Predefined status values:
--   -1 = rejected
--    0 = archived
--    1 = created
--    2 = approved

CREATE TABLE place (
    -- immutable
    id  INTEGER PRIMARY KEY,
    uid TEXT NOT NULL,
    lic TEXT NOT NULL, -- license
    -- mutable
    rev INTEGER NOT NULL, -- current revision = MAX(place_rev.rev)
    --
    UNIQUE (uid)
);

INSERT INTO place
SELECT rowid, id, license, version
FROM entries
WHERE current<>0
GROUP BY id
HAVING version=MAX(version);

-- Different revisions of a place
CREATE TABLE place_rev (
    -- immutable header
    id             INTEGER PRIMARY KEY,
    rev            INTEGER NOT NULL,
    place_id       INTEGER NOT NULL,
    created_at     INTEGER NOT NULL,
    created_by     INTEGER,
    -- mutable header
    status         INTEGER NOT NULL,
    -- immutable body
    title          TEXT NOT NULL,
    description    TEXT NOT NULL,
    lat            FLOAT NOT NULL,
    lon            FLOAT NOT NULL,
    street         TEXT,
    zip            TEXT,
    city           TEXT,
    country        TEXT,
    email          TEXT,
    phone          TEXT,
    homepage       TEXT,
    image_url      TEXT,
    image_link_url TEXT,
    --
    UNIQUE (place_id, rev),
    FOREIGN KEY (place_id) REFERENCES place(id),
    FOREIGN KEY (created_by) REFERENCES users(id)
);

INSERT INTO place_rev SELECT
entries.rowid,
entries.version, -- rev
place.id,
entries.created, -- created_at
NULL, -- created_by -> user_id
1, -- status = created (no archived entries yet!)
entries.title,
entries.description,
entries.lat,
entries.lng,
entries.street,
entries.zip,
entries.city,
entries.country,
entries.email,
entries.telephone,
entries.homepage,
entries.image_url,
entries.image_link_url
FROM entries
JOIN place ON entries.id=place.uid;

CREATE TABLE place_rev_status_log (
    id           INTEGER PRIMARY KEY,
    place_rev_id INTEGER NOT NULL,
    created_at   INTEGER NOT NULL,
    created_by   INTEGER,
    status       INTEGER NOT NULL,
    context      TEXT, -- any technical or system context, e.g. client IP address, ...
    notes        TEXT, -- human-written informational notes
    --
    FOREIGN KEY (place_rev_id) REFERENCES place_rev(id),
    FOREIGN KEY (created_by) REFERENCES users(id)
);

INSERT INTO place_rev_status_log SELECT
id,
id,
created_at,
created_by,
status,
NULL,
NULL
FROM place_rev;

CREATE TABLE place_rev_tag (
    place_rev_id INTEGER NOT NULL,
    tag          TEXT NOT NULL,
    --
    PRIMARY KEY (place_rev_id, tag),
    FOREIGN KEY (place_rev_id) REFERENCES place_rev(id)
);

INSERT OR IGNORE INTO place_rev_tag SELECT
place_rev.id,
trim(entry_tag_relations.tag_id)
FROM entry_tag_relations
JOIN place ON entry_tag_relations.entry_id=place.uid
JOIN place_rev ON place_rev.place_id=place.id AND place_rev.rev=entry_tag_relations.entry_version;

INSERT OR IGNORE INTO place_rev_tag SELECT
place_rev.id,
'non-profit'
FROM entry_category_relations
JOIN place ON entry_category_relations.entry_id=place.uid
JOIN place_rev ON place_rev.place_id=place.id AND place_rev.rev=entry_category_relations.entry_version
WHERE entry_category_relations.category_id='2cd00bebec0c48ba9db761da48678134';

INSERT OR IGNORE INTO place_rev_tag SELECT
place_rev.id,
'commercial'
FROM entry_category_relations
JOIN place ON entry_category_relations.entry_id=place.uid
JOIN place_rev ON place_rev.place_id=place.id AND place_rev.rev=entry_category_relations.entry_version
WHERE entry_category_relations.category_id='77b3c33a92554bcf8e8c2c86cedd6f6f';

-- Ratings apply to all revisions of a place
CREATE TABLE place_rating (
    -- immutable header
    id          INTEGER PRIMARY KEY,
    uid         TEXT NOT NULL,
    place_id    INTEGER NOT NULL,
    created_at  INTEGER NOT NULL,
    created_by  INTEGER,
    -- mutable header
    archived_at INTEGER,
    archived_by INTEGER,
    -- immutable body
    title       TEXT NOT NULL,
    value       INTEGER NOT NULL,
    context     TEXT NOT NULL,
    source      TEXT,
    --
    UNIQUE (uid),
    FOREIGN KEY (place_id) REFERENCES place(id),
    FOREIGN KEY (created_by) REFERENCES users(id),
    FOREIGN KEY (archived_by) REFERENCES users(id)
);

INSERT INTO place_rating SELECT
ratings.rowid,
ratings.id,
place.id,
ratings.created, -- created_at
NULL, -- created_by
ratings.archived, -- archived_at
NULL, -- archived_by
ratings.title,
ratings.value,
ratings.context,
ratings.source
FROM ratings
JOIN place ON place.uid=ratings.entry_id;

CREATE TABLE place_rating_comment (
    -- immutable header
    id              INTEGER PRIMARY KEY,
    uid             TEXT NOT NULL,
    rating_id INTEGER NOT NULL,
    created_at      INTEGER NOT NULL,
    created_by      INTEGER,
    -- mutable header
    archived_at     INTEGER,
    archived_by     INTEGER,
    -- immutable body
    text            TEXT NOT NULL,
    --
    UNIQUE (uid),
    FOREIGN KEY (rating_id) REFERENCES place_rating(id),
    FOREIGN KEY (created_by) REFERENCES users(id),
    FOREIGN KEY (archived_by) REFERENCES users(id)
);

INSERT INTO place_rating_comment SELECT
comments.rowid,
comments.id,
place_rating.id,
comments.created, -- created_at
NULL, -- created_by
comments.archived, -- archived_at
NULL, -- archived_by
comments.text
FROM comments
JOIN place_rating ON place_rating.uid=comments.rating_id;

DROP TABLE comments;
DROP TABLE ratings;
DROP TABLE entry_tag_relations;
DROP TABLE entry_category_relations;
DROP TABLE entries;
DROP TABLE categories;

CREATE INDEX place_rev_idx_created_at ON place_rev (created_at);
CREATE INDEX place_rev_idx_created_by ON place_rev (created_by);
CREATE INDEX place_rev_status_log_idx_created_at ON place_rev_status_log (created_at);
CREATE INDEX place_rev_status_log_idx_created_by ON place_rev_status_log (created_by);
CREATE INDEX place_rev_tag_idx_tag ON place_rev_tag (tag);
CREATE INDEX rating_idx_place_id ON place_rating (place_id);
CREATE INDEX rating_idx_created_at ON place_rating (created_at);
CREATE INDEX rating_idx_created_by ON place_rating (created_by);
CREATE INDEX rating_idx_archived_at ON place_rating (archived_at);
CREATE INDEX rating_idx_archived_by ON place_rating (archived_by);
CREATE INDEX place_rating_comment_idx_rating_id ON place_rating_comment (rating_id);
CREATE INDEX place_rating_comment_idx_created_at ON place_rating_comment (created_at);
CREATE INDEX place_rating_comment_idx_created_by ON place_rating_comment (created_by);
CREATE INDEX place_rating_comment_idx_archived_at ON place_rating_comment (archived_at);
CREATE INDEX place_rating_comment_idx_archived_by ON place_rating_comment (archived_by);

PRAGMA foreign_keys = ON;
