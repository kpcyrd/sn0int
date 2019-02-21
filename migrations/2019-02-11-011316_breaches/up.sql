CREATE TABLE breaches (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    value VARCHAR NOT NULL,
    unscoped BOOLEAN DEFAULT 0 NOT NULL,
    CONSTRAINT breach_unique UNIQUE (value)
);

CREATE TABLE breach_emails (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    breach_id INTEGER NOT NULL,
    email_id INTEGER NOT NULL,
    password VARCHAR,
    FOREIGN KEY(breach_id) REFERENCES breaches(id) ON DELETE CASCADE,
    FOREIGN KEY(email_id) REFERENCES emails(id) ON DELETE CASCADE,
    CONSTRAINT breach_emails_unique UNIQUE (breach_id, email_id, password)
);
