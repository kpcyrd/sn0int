DROP TRIGGER tsvectorupdate ON modules;
DROP FUNCTION modules_vector_update();

ALTER TABLE modules DROP COLUMN search_vector;
