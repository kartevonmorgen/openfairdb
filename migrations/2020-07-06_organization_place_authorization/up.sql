CREATE TABLE organization (
    rowid     INTEGER PRIMARY KEY NOT NULL,
    --
    id        TEXT NOT NULL,
    name      TEXT NOT NULL,
    api_token TEXT NOT NULL,
    --
    UNIQUE (id)
);

INSERT INTO organization
SELECT rowid, id, name, api_token
FROM organizations;

INSERT OR IGNORE INTO tags
SELECT DISTINCT tag_id
FROM org_tag_relations;

CREATE TABLE organization_tag_owned (
    rowid     INTEGER PRIMARY KEY NOT NULL,
    --
    org_rowid INTEGER NOT NULL,
    owned_tag TEXT NOT NULL,
    --
    UNIQUE (org_rowid, owned_tag),
    FOREIGN KEY (org_rowid) REFERENCES organization(rowid)
);

CREATE INDEX organization_tag_owned_idx_tag ON organization_tag_owned (owned_tag);

INSERT INTO organization_tag_owned
SELECT old.rowid as rowid, org.rowid as org_rowid, old.tag_id as tag_id
FROM org_tag_relations old
JOIN organization org
ON org.id=old.org_id;

DROP TABLE org_tag_relations;
DROP TABLE organizations;

-- Pending authorizations by organizations for selected/observed places
CREATE TABLE organization_place_last_authorized (
    rowid        INTEGER PRIMARY KEY,
    --
    org_rowid    INTEGER NOT NULL,
    place_rowid  INTEGER NOT NULL,
    --
    created_at   INTEGER NOT NULL,
    --
    last_authorized_place_rev           INTEGER, -- last authorized revision number or NULL if the place has not been authorized yet
    last_authorized_place_review_status INTEGER, -- current review status upon authorization or NULL if the place has not been authorized yet
    --
    UNIQUE (org_rowid, place_rowid),
    FOREIGN KEY (org_rowid) REFERENCES organizations(rowid),
    FOREIGN KEY (place_rowid) REFERENCES place(rowid)
);
