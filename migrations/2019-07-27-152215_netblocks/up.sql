CREATE TABLE netblocks (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    family VARCHAR NOT NULL,
    value VARCHAR NOT NULL,
    unscoped BOOLEAN DEFAULT 0 NOT NULL,
    asn INTEGER,
    as_org VARCHAR,
    description VARCHAR,
    CONSTRAINT netblock_unique UNIQUE (value)
);
