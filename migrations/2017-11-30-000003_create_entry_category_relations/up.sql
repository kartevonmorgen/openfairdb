CREATE TABLE entry_category_relations (
    entry_id    TEXT NOT NULL,
    category_id TEXT NOT NULL,
    PRIMARY KEY (entry_id, category_id)
);
