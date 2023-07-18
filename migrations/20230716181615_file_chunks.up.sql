-- Add up migration script here
CREATE TABLE IF NOT EXISTS file_chunks (
    id TEXT PRIMARY KEY,
    parent_file_id TEXT NOT NULL,
    idx smallint NOT NULL,
    content TEXT NOT NULL,
    content_embedding vector(384)
);
