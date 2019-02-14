CREATE TABLE accounts (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    value VARCHAR NOT NULL,
    service VARCHAR NOT NULL,
    username VARCHAR NOT NULL,
    displayname VARCHAR,
    email VARCHAR,
    url VARCHAR,
    last_seen DATETIME,
    unscoped BOOLEAN DEFAULT 0 NOT NULL,
    CONSTRAINT account_unique UNIQUE (value)
);
