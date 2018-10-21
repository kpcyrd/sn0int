CREATE TABLE domains (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    value VARCHAR NOT NULL,
    CONSTRAINT domain_unique UNIQUE (value)
);

CREATE TABLE subdomains (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    domain_id INTEGER NOT NULL,
    value VARCHAR NOT NULL,
    FOREIGN KEY(domain_id) REFERENCES domains(id),
    CONSTRAINT subdomain_unique UNIQUE (value)
);

/* family maybe not needed */
CREATE TABLE ipaddrs (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    family VARCHAR NOT NULL,
    value VARCHAR NOT NULL,
    CONSTRAINT ipaddr_unique UNIQUE (value)
);

CREATE TABLE subdomain_ipaddrs (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    subdomain_id INTEGER NOT NULL,
    ip_addr_id INTEGER NOT NULL,
    FOREIGN KEY(subdomain_id) REFERENCES subdomains(id),
    FOREIGN KEY(ip_addr_id) REFERENCES ipaddrs(id),
    CONSTRAINT subdomain_ipaddr_unique UNIQUE (subdomain_id, ip_addr_id)
);

CREATE TABLE urls (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    subdomain_id INTEGER NOT NULL,
    value VARCHAR NOT NULL,
    status INTEGER,
    body BLOB,
    FOREIGN KEY(subdomain_id) REFERENCES subdomains(id),
    CONSTRAINT url_unique UNIQUE (value)
);
