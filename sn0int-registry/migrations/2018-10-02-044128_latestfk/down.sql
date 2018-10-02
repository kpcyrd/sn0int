ALTER TABLE modules DROP COLUMN latest;
ALTER TABLE modules ADD COLUMN latest SERIAL;
ALTER TABLE modules
    ADD CONSTRAINT latestfk
    FOREIGN KEY (latest)
    REFERENCES releases (id) MATCH FULL;
