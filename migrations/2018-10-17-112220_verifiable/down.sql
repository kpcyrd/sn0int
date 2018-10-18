PRAGMA foreign_keys=off;

-- subdomains

ALTER TABLE subdomains RENAME TO _subdomains_old;

CREATE TABLE subdomains (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    domain_id INTEGER NOT NULL,
    value VARCHAR NOT NULL,
    unscoped BOOLEAN DEFAULT 0 NOT NULL,
    FOREIGN KEY(domain_id) REFERENCES domains(id),
    CONSTRAINT subdomain_unique UNIQUE (value)
);

INSERT INTO subdomains (id, domain_id, value, unscoped)
  SELECT id, domain_id, value, unscoped
  FROM _subdomains_old;

DROP TABLE _subdomains_old;

-- emails

ALTER TABLE emails RENAME TO _emails_old;

CREATE TABLE emails (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    value VARCHAR NOT NULL,
    unscoped BOOLEAN DEFAULT 0 NOT NULL,
    CONSTRAINT emails_unique UNIQUE (value)
);

INSERT INTO emails (id, value, unscoped)
  SELECT id, value, unscoped
  FROM _emails_old;

DROP TABLE _emails_old;

-- urls

ALTER TABLE urls RENAME TO _urls_old;

CREATE TABLE urls (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    subdomain_id INTEGER NOT NULL,
    value VARCHAR NOT NULL,
    status INTEGER,
    body BLOB,
    unscoped BOOLEAN DEFAULT 0 NOT NULL,
    FOREIGN KEY(subdomain_id) REFERENCES subdomains(id),
    CONSTRAINT url_unique UNIQUE (value)
);

INSERT INTO urls (id, subdomain_id, value, status, body, unscoped)
  SELECT id, subdomain_id, value, status, body, unscoped
  FROM _urls_old;

DROP TABLE _urls_old;

PRAGMA foreign_keys=on;
