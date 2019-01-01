CREATE TABLE networks (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    value VARCHAR NOT NULL,
    unscoped BOOLEAN DEFAULT 0 NOT NULL,
    latitude FLOAT,
    longitude FLOAT,
    CONSTRAINT network_unique UNIQUE (value)
);

CREATE TABLE devices (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    value VARCHAR NOT NULL,
    name VARCHAR,
    hostname VARCHAR,
    vendor VARCHAR,
    unscoped BOOLEAN DEFAULT 0 NOT NULL,
    last_seen DATETIME,
    CONSTRAINT device_unique UNIQUE (value)
);

CREATE TABLE network_devices (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    network_id INTEGER NOT NULL,
    device_id INTEGER NOT NULL,
    ipaddr VARCHAR,
    last_seen DATETIME,
    FOREIGN KEY(network_id) REFERENCES networks(id) ON DELETE CASCADE,
    FOREIGN KEY(device_id) REFERENCES devices(id) ON DELETE CASCADE,
    CONSTRAINT network_device_unique UNIQUE (network_id, device_id)
);
