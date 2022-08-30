CREATE TABLE email_token_credentials (
    id          INTEGER PRIMARY KEY,
    expires_at  INTEGER NOT NULL,
    username    TEXT NOT NULL,
    email       TEXT NOT NULL,
    nonce       TEXT NOT NULL,
    UNIQUE      (username),
    UNIQUE      (nonce),
    FOREIGN KEY (username) REFERENCES users(username)
);
