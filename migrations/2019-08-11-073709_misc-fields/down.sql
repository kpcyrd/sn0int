PRAGMA foreign_keys=off;

-- accounts
CREATE TABLE _accounts_new (
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

INSERT INTO _accounts_new (id, value, service, username, displayname, email, url, last_seen, unscoped)
    SELECT id, value, service, username, displayname, email, url, last_seen, unscoped
    FROM accounts;

DROP TABLE accounts;
ALTER TABLE _accounts_new RENAME TO accounts;

-- networks
CREATE TABLE _networks_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    value VARCHAR NOT NULL,
    unscoped BOOLEAN DEFAULT 0 NOT NULL,
    latitude FLOAT,
    longitude FLOAT,
    CONSTRAINT network_unique UNIQUE (value)
);

INSERT INTO _networks_new (id, value, unscoped, latitude, longitude)
    SELECT id, value, unscoped, latitude, longitude
    FROM networks;

DROP TABLE networks;
ALTER TABLE _networks_new RENAME TO networks;

PRAGMA foreign_keys=on;
