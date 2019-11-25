-- Normalize homepage and email
UPDATE entries
SET (homepage, email) = (trim(homepage), trim(email));

-- Exchange email/homepage if URL contains '@'
UPDATE entries
SET (homepage, email) = (email, homepage)
WHERE homepage LIKE '%@%'
AND (email IS NULL OR email NOT LIKE '%@%');

-- Fix incomplete URLs
UPDATE entries
SET homepage = 'https://' || homepage
WHERE NOT (homepage IS NULL OR homepage LIKE 'http%');

-- Disable fk constraints temporarily to allow inserting NULL references
-- into new tables with foreign keys.
PRAGMA foreign_keys = OFF;

CREATE TABLE place (
    id  INTEGER PRIMARY KEY,
    --
    current_rev INTEGER NOT NULL, -- latest revision (mutable) from place_revision
    --
    uid TEXT NOT NULL,
    lic TEXT NOT NULL, -- license
    --
    UNIQUE (uid)
);

INSERT INTO place
SELECT rowid, version, id, trim(license)
FROM entries
WHERE current<>0
GROUP BY id
HAVING version=MAX(version);

-- Different revisions of a place
CREATE TABLE place_revision (
    id             INTEGER PRIMARY KEY,
    parent_id      INTEGER NOT NULL,
    --
    rev            INTEGER NOT NULL,
    created_at     INTEGER NOT NULL,
    created_by     INTEGER,
    --
    current_status INTEGER NOT NULL, -- latest status (mutable) from place_revision_review
    --
    title          TEXT NOT NULL,
    desc           TEXT NOT NULL,
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
    UNIQUE (parent_id, rev),
    FOREIGN KEY (parent_id) REFERENCES place(id),
    FOREIGN KEY (created_by) REFERENCES users(id)
);

INSERT INTO place_revision SELECT
entries.rowid,
place.id, -- parent_id
entries.version, -- rev
entries.created * 1000, -- created_at (seconds -> milliseconds)
NULL, -- created_by -> user_id
1, -- current_status = created (no archived entries yet!)
trim(entries.title),
trim(entries.description),
entries.lat,
entries.lng,
trim(entries.street),
trim(entries.zip),
trim(entries.city),
trim(entries.country),
trim(entries.email),
trim(entries.telephone),
trim(entries.homepage),
trim(entries.image_url),
trim(entries.image_link_url)
FROM entries
JOIN place ON entries.id=place.uid
WHERE archived IS NULL; -- no archived entries yet (otherwise ignored)!

CREATE TABLE place_revision_review (
    id         INTEGER PRIMARY KEY,
    parent_id  INTEGER NOT NULL,
    --
    rev        INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    created_by INTEGER,
    --
    status     INTEGER NOT NULL,
    context    TEXT, -- system context, e.g. client IP address, ...
    memo       TEXT, -- human-written textual memo
    --
    UNIQUE (parent_id, rev),
    FOREIGN KEY (parent_id) REFERENCES place_revision(id),
    FOREIGN KEY (created_by) REFERENCES users(id)
);

INSERT INTO place_revision_review SELECT
id,
id, -- parent_id
0,
created_at,
created_by,
current_status,
NULL,
NULL
FROM place_revision;

CREATE TABLE place_revision_tag (
    parent_id INTEGER NOT NULL,
    tag       TEXT NOT NULL,
    PRIMARY KEY (parent_id, tag),
    FOREIGN KEY (parent_id) REFERENCES place_revision(id)
);

INSERT OR IGNORE INTO place_revision_tag SELECT
place_revision.id,
trim(entry_tag_relations.tag_id)
FROM entry_tag_relations
JOIN place ON place.uid=entry_tag_relations.entry_id
JOIN place_revision ON place_revision.parent_id=place.id AND place_revision.rev=entry_tag_relations.entry_version;

INSERT OR IGNORE INTO place_revision_tag SELECT
place_revision.id,
'non-profit'
FROM entry_category_relations
JOIN place ON place.uid=entry_category_relations.entry_id
JOIN place_revision ON place_revision.parent_id=place.id AND place_revision.rev=entry_category_relations.entry_version
WHERE entry_category_relations.category_id='2cd00bebec0c48ba9db761da48678134';

INSERT OR IGNORE INTO place_revision_tag SELECT
place_revision.id,
'commercial'
FROM entry_category_relations
JOIN place ON place.uid=entry_category_relations.entry_id
JOIN place_revision ON place_revision.parent_id=place.id AND place_revision.rev=entry_category_relations.entry_version
WHERE entry_category_relations.category_id='77b3c33a92554bcf8e8c2c86cedd6f6f';

-- Ratings apply to all revisions of a place, i.e. place
CREATE TABLE place_rating (
    id          INTEGER PRIMARY KEY,
    parent_id   INTEGER NOT NULL,
    --
    created_at  INTEGER NOT NULL,
    created_by  INTEGER,
    archived_at INTEGER,
    archived_by INTEGER,
    --
    uid         TEXT NOT NULL,
    title       TEXT NOT NULL,
    value       INTEGER NOT NULL,
    context     TEXT NOT NULL,
    source      TEXT,
    --
    UNIQUE (uid),
    FOREIGN KEY (parent_id) REFERENCES place(id),
    FOREIGN KEY (created_by) REFERENCES users(id),
    FOREIGN KEY (archived_by) REFERENCES users(id)
);

INSERT INTO place_rating SELECT
ratings.rowid, -- id
place.id, -- parent_id
ratings.created, -- created_at
NULL, -- created_by
ratings.archived, -- archived_at
NULL, -- archived_by
ratings.id, -- uid
trim(ratings.title),
ratings.value,
ratings.context,
trim(ratings.source)
FROM ratings
JOIN place ON place.uid=ratings.entry_id;

CREATE TABLE place_rating_comment (
    id          INTEGER PRIMARY KEY,
    parent_id   INTEGER NOT NULL,
    --
    created_at  INTEGER NOT NULL,
    created_by  INTEGER,
    archived_at INTEGER,
    archived_by INTEGER,
    --
    uid         TEXT NOT NULL,
    text        TEXT NOT NULL,
    --
    UNIQUE (uid),
    FOREIGN KEY (parent_id) REFERENCES place_rating(id),
    FOREIGN KEY (created_by) REFERENCES users(id),
    FOREIGN KEY (archived_by) REFERENCES users(id)
);

INSERT INTO place_rating_comment SELECT
comments.rowid,
place_rating.id, -- parent_id
comments.created, -- created_at
NULL, -- created_by
comments.archived, -- archived_at
NULL, -- archived_by
comments.id, -- uid
trim(comments.text)
FROM comments
JOIN place_rating ON place_rating.uid=comments.rating_id;

DROP TABLE comments;
DROP TABLE ratings;
DROP TABLE entry_tag_relations;
DROP TABLE entry_category_relations;
DROP TABLE entries;
DROP TABLE categories;

CREATE INDEX place_revision_idx_created_at ON place_revision (created_at);
CREATE INDEX place_revision_idx_created_by ON place_revision (created_by);
CREATE INDEX place_revision_review_idx_created_at ON place_revision_review (created_at);
CREATE INDEX place_revision_review_idx_created_by ON place_revision_review (created_by);
CREATE INDEX place_revision_tag_idx_tag ON place_revision_tag (tag);
CREATE INDEX place_rating_idx_parent_id ON place_rating (parent_id);
CREATE INDEX place_rating_idx_created_at ON place_rating (created_at);
CREATE INDEX place_rating_idx_created_by ON place_rating (created_by);
CREATE INDEX place_rating_idx_archived_at ON place_rating (archived_at);
CREATE INDEX place_rating_idx_archived_by ON place_rating (archived_by);
CREATE INDEX place_rating_comment_idx_parent_id ON place_rating_comment (parent_id);
CREATE INDEX place_rating_comment_idx_created_at ON place_rating_comment (created_at);
CREATE INDEX place_rating_comment_idx_created_by ON place_rating_comment (created_by);
CREATE INDEX place_rating_comment_idx_archived_at ON place_rating_comment (archived_at);
CREATE INDEX place_rating_comment_idx_archived_by ON place_rating_comment (archived_by);

PRAGMA foreign_keys = ON;
