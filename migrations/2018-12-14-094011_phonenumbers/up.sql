CREATE TABLE phonenumbers (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    value VARCHAR NOT NULL,
    name VARCHAR,
    unscoped BOOLEAN DEFAULT 0 NOT NULL,
    valid BOOLEAN,
    last_online DATETIME,
    country VARCHAR,
    carrier VARCHAR,
    line VARCHAR,
    is_ported BOOLEAN,
    last_ported DATETIME,
    caller_name VARCHAR,
    caller_type VARCHAR,
    CONSTRAINT phonenumber_unique UNIQUE (value)
);
