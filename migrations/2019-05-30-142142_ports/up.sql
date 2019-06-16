CREATE TABLE ports (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    ip_addr_id INTEGER NOT NULL,
    value VARCHAR NOT NULL,
    ip_addr VARCHAR NOT NULL,
    port INTEGER NOT NULL,
    protocol VARCHAR NOT NULL,
    status VARCHAR NOT NULL,
    unscoped BOOLEAN DEFAULT 0 NOT NULL,

    banner VARCHAR,
    service VARCHAR,
    version VARCHAR,

    FOREIGN KEY(ip_addr_id) REFERENCES ipaddrs(id) ON DELETE CASCADE,
    CONSTRAINT port_unique UNIQUE (value)
);
