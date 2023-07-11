-- Add up migration script here

    CREATE TABLE IF NOT EXISTS directories (
        id SERIAL PRIMARY KEY,
        dirpath TEXT NOT NULL,
        project BOOLEAN,
        parent_dir_id INTEGER,
        CHECK ((NOT project) OR (project AND parent_dir_id IS NULL))
    );
