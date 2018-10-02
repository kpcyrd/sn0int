ALTER TABLE modules
ADD CONSTRAINT modules_uniq UNIQUE (author, name);
ALTER TABLE releases
ADD CONSTRAINT releases_uniq UNIQUE (module_id, version);
