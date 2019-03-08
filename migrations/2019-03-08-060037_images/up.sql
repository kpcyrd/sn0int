CREATE TABLE images (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    value VARCHAR NOT NULL,

    filename VARCHAR,
    mime VARCHAR,
    width INT,
    height INT,
    created DATETIME,

    latitude FLOAT,
    longitude FLOAT,

    nudity FLOAT,
    ahash VARCHAR,
    dhash VARCHAR,
    phash VARCHAR,

    unscoped BOOLEAN DEFAULT 0 NOT NULL,
    CONSTRAINT image_unique UNIQUE (value)
);
