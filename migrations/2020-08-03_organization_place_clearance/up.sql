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

CREATE TABLE organization_tag (
    rowid     INTEGER PRIMARY KEY NOT NULL,
    --
    org_rowid INTEGER NOT NULL,
    tag_label TEXT NOT NULL,
    --
    tag_moderation_flags SMALLINT NOT NULL,
    --
    UNIQUE (org_rowid, tag_label),
    FOREIGN KEY (org_rowid) REFERENCES organization(rowid)
    -- no FK for tag_label (may not yet exist)
);

CREATE INDEX organization_tag_idx_tag_label ON organization_tag(tag_label);

INSERT INTO organization_tag
SELECT old.rowid as rowid, org.rowid as org_rowid, old.tag_id as tag_label, 0 as tag_moderation_flags
FROM org_tag_relations old
JOIN organization org
ON org.id=old.org_id;

DROP TABLE org_tag_relations;
DROP TABLE organizations;

-- Pending authorization/approval of places by organizations
CREATE TABLE organization_place_clearance (
    rowid        INTEGER PRIMARY KEY,
    --
    org_rowid    INTEGER NOT NULL,
    place_rowid  INTEGER NOT NULL,
    --
    created_at   INTEGER NOT NULL,
    --
    last_cleared_revision INTEGER, -- last cleared revision number or NULL if the place is new and has not been cleared yet
    --
    UNIQUE (org_rowid, place_rowid),
    FOREIGN KEY (org_rowid) REFERENCES organizations(rowid),
    FOREIGN KEY (place_rowid) REFERENCES place(rowid)
);
