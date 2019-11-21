CREATE TABLE cryptoaddrs (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    value VARCHAR NOT NULL,
    currency VARCHAR,
    denominator INTEGER,
    balance BIGINT,
    received BIGINT,
    first_seen DATETIME,
    last_withdrawal DATETIME,
    unscoped BOOLEAN DEFAULT 0 NOT NULL,
    description VARCHAR,
    CONSTRAINT netblock_unique UNIQUE (value)
);
