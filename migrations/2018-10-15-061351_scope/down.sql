PRAGMA foreign_keys=off;

-- domains

ALTER TABLE domains RENAME TO _domains_old;

CREATE TABLE domains (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    value VARCHAR NOT NULL,
    CONSTRAINT domain_unique UNIQUE (value)
);

INSERT INTO domains (id, value)
  SELECT id, value
  FROM _domains_old;

DROP TABLE _domains_old;

-- subdomains

ALTER TABLE subdomains RENAME TO _subdomains_old;

CREATE TABLE subdomains (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    domain_id INTEGER NOT NULL,
    value VARCHAR NOT NULL,
    FOREIGN KEY(domain_id) REFERENCES domains(id),
    CONSTRAINT subdomain_unique UNIQUE (value)
);

INSERT INTO subdomains (id, domain_id, value)
  SELECT id, domain_id, value
  FROM _subdomains_old;

DROP TABLE _subdomains_old;

-- ipaddrs

ALTER TABLE ipaddrs RENAME TO _ipaddrs_old;

CREATE TABLE ipaddrs (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    family VARCHAR NOT NULL,
    value VARCHAR NOT NULL,
    CONSTRAINT ipaddr_unique UNIQUE (value)
);

INSERT INTO ipaddrs (id, family, value)
  SELECT id, family, value
  FROM _ipaddrs_old;

DROP TABLE _ipaddrs_old;

-- urls

ALTER TABLE urls RENAME TO _urls_old;

CREATE TABLE urls (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    subdomain_id INTEGER NOT NULL,
    value VARCHAR NOT NULL,
    status INTEGER,
    body BLOB,
    FOREIGN KEY(subdomain_id) REFERENCES subdomains(id),
    CONSTRAINT url_unique UNIQUE (value)
);

INSERT INTO urls (id, subdomain_id, value, status, body)
  SELECT id, subdomain_id, value, status, body
  FROM _urls_old;

DROP TABLE _urls_old;

-- emails

ALTER TABLE emails RENAME TO _emails_old;

CREATE TABLE emails (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    value VARCHAR NOT NULL,
    CONSTRAINT domain_unique UNIQUE (value)
);

INSERT INTO emails (id, value)
  SELECT id, value
  FROM _emails_old;

DROP TABLE _emails_old;

PRAGMA foreign_keys=on;
