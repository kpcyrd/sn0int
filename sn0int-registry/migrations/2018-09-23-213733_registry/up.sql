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
    token VARCHAR PRIMARY KEY,
    author VARCHAR NOT NULL,
    created_at TIMESTAMP NOT NULL,
    used_at TIMESTAMP NOT NULL,
    oauth VARCHAR NOT NULL
);
