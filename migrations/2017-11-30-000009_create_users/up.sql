CREATE TABLE users (
    username        TEXT PRIMARY KEY NOT NULL,
    password        TEXT    NOT NULL,
    email           TEXT    NOT NULL,
    email_confirmed BOOLEAN NOT NULL
);
