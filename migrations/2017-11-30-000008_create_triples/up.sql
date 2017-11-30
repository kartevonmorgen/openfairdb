CREATE TABLE triples (
    subject_id    TEXT NOT NULL,
    subject_type  TEXT NOT NULL,
    predicate     TEXT NOT NULL,
    object_id     TEXT NOT NULL,
    object_type   TEXT NOT NULL,
    PRIMARY KEY (subject_id, predicate, object_id)
);
