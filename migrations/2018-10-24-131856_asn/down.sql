PRAGMA foreign_keys=off;

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
    CONSTRAINT ipaddr_unique UNIQUE (value)
);

INSERT INTO ipaddrs (id, family, value, unscoped, continent, continent_code, country, country_code, city, latitude, longitude)
  SELECT id, family, value, unscoped, continent, continent_code, country, country_code, city, latitude, longitude
  FROM _ipaddrs_old;

DROP TABLE _ipaddrs_old;

PRAGMA foreign_keys=on;
