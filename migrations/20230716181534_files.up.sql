-- Add up migration script here
CREATE TABLE IF NOT EXISTS files (
    id TEXT PRIMARY KEY,
    thread_name TEXT,
    filepath TEXT NOT NULL,
    parent_dir_path TEXT NOT NULL,
    summary TEXT NOT NULL,
    summary_embedding vector(384)
);
