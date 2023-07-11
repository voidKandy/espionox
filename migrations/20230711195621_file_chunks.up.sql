-- Add up migration script here
    CREATE TABLE IF NOT EXISTS files (
        id SERIAL PRIMARY KEY,
        parent_file_id SERIAL NOT NULL,
        index INTEGER NOT NULL,
        content TEXT NOT NULL,
        content_embedding vector(384)
    );
