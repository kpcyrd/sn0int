PRAGMA foreign_keys=off;

CREATE TABLE _ipaddrs_new (
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

INSERT INTO _ipaddrs_new (id, family, value, unscoped, continent, continent_code, city, latitude, longitude, asn, as_org)
  SELECT id, family, value, unscoped, continent, continent_code, city, latitude, longitude, asn, as_org
  FROM ipaddrs;

DROP TABLE ipaddrs;
ALTER TABLE _ipaddrs_new RENAME TO ipaddrs;

PRAGMA foreign_keys=on;
