ALTER TABLE organization_place_clearance RENAME TO organization_place_clearance_tmp;

-- Pending authorization/approval of places by organizations
CREATE TABLE organization_place_clearance (
    rowid        INTEGER PRIMARY KEY,
    --
    org_rowid    INTEGER NOT NULL,
    place_rowid  INTEGER NOT NULL,
    --
    created_at            INTEGER NOT NULL,
    last_cleared_revision INTEGER, -- last cleared revision number or NULL if the place is new and has not been cleared yet
    --
    UNIQUE (org_rowid, place_rowid),
    FOREIGN KEY (org_rowid) REFERENCES organization(rowid),
    FOREIGN KEY (place_rowid) REFERENCES place(rowid)
);

INSERT INTO organization_place_clearance SELECT * FROM organization_place_clearance_tmp;
