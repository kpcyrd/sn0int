PRAGMA foreign_keys=off;

ALTER TABLE urls RENAME TO _urls_old;

CREATE TABLE urls (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    subdomain_id INTEGER NOT NULL,
    value VARCHAR NOT NULL,
    status INTEGER,
    body BLOB,
    unscoped BOOLEAN DEFAULT 0 NOT NULL,
    online BOOLEAN,
    title VARCHAR,
    redirect VARCHAR,
    FOREIGN KEY(subdomain_id) REFERENCES subdomains(id) ON DELETE CASCADE,
    CONSTRAINT url_unique UNIQUE (value)
);

INSERT INTO urls (id, subdomain_id, value, status, body, unscoped, online, title, redirect)
  SELECT id, subdomain_id, value, status, body, unscoped, online, title, redirect
  FROM _urls_old;

DROP TABLE _urls_old;

PRAGMA foreign_keys=on;
