-- Add up migration script here
CREATE TABLE IF NOT EXISTS io (
    id TEXT PRIMARY KEY,
    input TEXT,
    output TEXT
);
