ALTER TABLE modules ADD COLUMN search_vector TSVECTOR NOT NULL;

CREATE FUNCTION modules_vector_update() RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        new.search_vector = to_tsvector('pg_catalog.english',
            NEW.name || ' ' || NEW.author || ' ' || NEW.description
        );
    END IF;
    IF TG_OP = 'UPDATE' THEN
        IF NEW.description <> OLD.description THEN
            new.search_vector = to_tsvector('pg_catalog.english',
                NEW.name || ' ' || NEW.author || ' ' || NEW.description
            );
        END IF;
    END IF;
    RETURN NEW;
END
$$ LANGUAGE 'plpgsql';

CREATE TRIGGER tsvectorupdate BEFORE INSERT OR UPDATE ON modules
FOR EACH ROW EXECUTE PROCEDURE modules_vector_update();
