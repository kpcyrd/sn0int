ALTER TABLE modules ADD COLUMN source VARCHAR;

CREATE OR REPLACE FUNCTION modules_vector_update() RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        new.search_vector = to_tsvector('pg_catalog.english',
            NEW.name || ' ' || NEW.author || ' ' || COALESCE(NEW.source, '') || ' ' || NEW.description
        );
    END IF;
    IF TG_OP = 'UPDATE' THEN
        IF NEW.description <> OLD.description OR NEW.source <> OLD.source THEN
            new.search_vector = to_tsvector('pg_catalog.english',
                NEW.name || ' ' || NEW.author || ' ' || COALESCE(NEW.source, '') || ' ' || NEW.description
            );
        END IF;
    END IF;
    RETURN NEW;
END
$$ LANGUAGE 'plpgsql';
