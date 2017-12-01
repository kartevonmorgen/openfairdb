CREATE TABLE entry_category_relations (
    entry_id      TEXT NOT NULL,
    entry_version INTEGER NOT NULL,
    category_id   TEXT NOT NULL,
    PRIMARY KEY (entry_id, entry_version, category_id),
    FOREIGN KEY (entry_id, entry_version) REFERENCES entries(id,version)
);
