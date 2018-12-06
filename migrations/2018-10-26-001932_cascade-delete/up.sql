PRAGMA foreign_keys=off;

-- domains

ALTER TABLE domains RENAME TO _domains_old;

CREATE TABLE domains (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    value VARCHAR NOT NULL,
    unscoped BOOLEAN DEFAULT 0 NOT NULL,
    CONSTRAINT domain_unique UNIQUE (value)
);

INSERT INTO domains (id, value, unscoped)
    SELECT id, value, unscoped
    FROM _domains_old;

DROP TABLE _domains_old;

-- subdomains

ALTER TABLE subdomains RENAME TO _subdomains_old;

CREATE TABLE subdomains (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    domain_id INTEGER NOT NULL,
    value VARCHAR NOT NULL,
    unscoped BOOLEAN DEFAULT 0 NOT NULL,
    resolvable BOOLEAN,
    FOREIGN KEY(domain_id) REFERENCES domains(id) ON DELETE CASCADE,
    CONSTRAINT subdomain_unique UNIQUE (value)
);

INSERT INTO subdomains (id, domain_id, value, unscoped, resolvable)
    SELECT id, domain_id, value, unscoped, resolvable
    FROM _subdomains_old;

DROP TABLE _subdomains_old;

-- urls

ALTER TABLE urls RENAME TO _urls_old;

CREATE TABLE urls (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    subdomain_id INTEGER NOT NULL,
    value VARCHAR NOT NULL,
    status INTEGER,
    body BLOB,
    unscoped BOOLEAN DEFAULT 0 NOT NULL,
    online BOOLEAN,
    FOREIGN KEY(subdomain_id) REFERENCES subdomains(id) ON DELETE CASCADE,
    CONSTRAINT url_unique UNIQUE (value)
);

INSERT INTO urls (id, subdomain_id, value, status, body, unscoped, online)
    SELECT id, subdomain_id, value, status, body, unscoped, online
    FROM _urls_old;

DROP TABLE _urls_old;

-- emails

ALTER TABLE emails RENAME TO _emails_old;

CREATE TABLE emails (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    value VARCHAR NOT NULL,
    unscoped BOOLEAN DEFAULT 0 NOT NULL,
    valid BOOLEAN,
    CONSTRAINT email_unique UNIQUE (value)
);

INSERT INTO emails (id, value, unscoped, valid)
    SELECT id, value, unscoped, valid
    FROM _emails_old;

DROP TABLE _emails_old;

-- ipaddrs

ALTER TABLE ipaddrs RENAME TO _ipaddrs_old;

CREATE TABLE ipaddrs (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    family VARCHAR NOT NULL,
    value VARCHAR NOT NULL,
    unscoped BOOLEAN DEFAULT 0 NOT NULL,
    continent VARCHAR,
    continent_code VARCHAR,
    country VARCHAR,
    country_code VARCHAR,
    city VARCHAR,
    latitude FLOAT,
    longitude FLOAT,
    asn INTEGER,
    as_org VARCHAR,
    CONSTRAINT ipaddr_unique UNIQUE (value)
);

INSERT INTO ipaddrs (id, family, value, unscoped, continent, continent_code, country, country_code, city, latitude, longitude, asn, as_org)
  SELECT id, family, value, unscoped, continent, continent_code, country, country_code, city, latitude, longitude, asn, as_org
  FROM _ipaddrs_old;

DROP TABLE _ipaddrs_old;

-- subdomain_ipaddrs

ALTER TABLE subdomain_ipaddrs RENAME TO _subdomain_ipaddrs_old;

CREATE TABLE subdomain_ipaddrs (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    subdomain_id INTEGER NOT NULL,
    ip_addr_id INTEGER NOT NULL,
    FOREIGN KEY(subdomain_id) REFERENCES subdomains(id) ON DELETE CASCADE,
    FOREIGN KEY(ip_addr_id) REFERENCES ipaddrs(id) ON DELETE CASCADE,
    CONSTRAINT subdomain_ipaddr_unique UNIQUE (subdomain_id, ip_addr_id)
);

INSERT INTO subdomain_ipaddrs (id, subdomain_id, ip_addr_id)
    SELECT id, subdomain_id, ip_addr_id
    FROM _subdomain_ipaddrs_old;

DROP TABLE _subdomain_ipaddrs_old;

PRAGMA foreign_keys=on;
