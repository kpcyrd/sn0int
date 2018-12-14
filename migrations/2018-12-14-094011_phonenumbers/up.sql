CREATE TABLE phonenumbers (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    value VARCHAR NOT NULL,
    name VARCHAR,
    unscoped BOOLEAN DEFAULT 0 NOT NULL,
    valid BOOLEAN,
    CONSTRAINT phonenumber_unique UNIQUE (value)
);
