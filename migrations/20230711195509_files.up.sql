-- Add up migration script here
-- SENTENCE EMBEDDING MODEL CREATES VECTORS OF SIZE 384
    CREATE TABLE IF NOT EXISTS files (
        id SERIAL PRIMARY KEY,
        filepath TEXT NOT NULL,
        summary_embedding vector(384),
        parent_dir_id INTEGER
    );
