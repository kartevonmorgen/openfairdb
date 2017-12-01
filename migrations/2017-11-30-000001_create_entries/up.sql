CREATE TABLE entries (
    id          TEXT NOT NULL,
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
