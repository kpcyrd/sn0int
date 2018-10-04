CREATE INDEX modules_search_idx ON modules USING GIN(search_vector);
