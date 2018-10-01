CREATE TABLE modules (
    id SERIAL PRIMARY KEY,
    author VARCHAR NOT NULL,
    name VARCHAR NOT NULL,
    description TEXT NOT NULL,
    search_tokens TSVECTOR,
    latest SERIAL
    -- latest SERIAL references releases(id)
);

CREATE TABLE releases (
    id SERIAL PRIMARY KEY,
    module_id SERIAL NOT NULL references modules(id),
    version VARCHAR NOT NULL,
    downloads int NOT NULL DEFAULT 0
);
ALTER TABLE modules
    ADD CONSTRAINT latestfk
    FOREIGN KEY (latest)
    REFERENCES releases (id) MATCH FULL;

CREATE TABLE auth_tokens (
    id VARCHAR PRIMARY KEY,
    author VARCHAR NOT NULL,
    access_token VARCHAR NOT NULL
);
