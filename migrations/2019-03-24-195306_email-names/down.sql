PRAGMA foreign_keys=off;

CREATE TABLE _emails_new (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    value VARCHAR NOT NULL,
    unscoped BOOLEAN DEFAULT 0 NOT NULL,
    valid BOOLEAN,
    CONSTRAINT email_unique UNIQUE (value)
);

INSERT INTO _emails_new (id, value, unscoped, valid)
    SELECT id, value, unscoped, valid
    FROM emails;

DROP TABLE emails;
ALTER TABLE _emails_new RENAME TO emails;

PRAGMA foreign_keys=on;
