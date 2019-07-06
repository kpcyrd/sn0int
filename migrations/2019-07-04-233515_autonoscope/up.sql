-- Your SQL goes here
CREATE TABLE autonoscope (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    object VARCHAR NOT NULL,
    value VARCHAR NOT NULL,
    scoped BOOLEAN NOT NULL,
    CONSTRAINT autonoscope_unique UNIQUE (object, value)
);
