-- Add up migration script here
CREATE TABLE IF NOT EXISTS errors (
    id TEXT PRIMARY KEY,
    context_id SERIAL NOT NULL,
    content TEXT NOT NULL,
    content_embedding vector(384)
);
