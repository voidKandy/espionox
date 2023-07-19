-- Add up migration script here
CREATE TABLE IF NOT EXISTS contexts (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL
);
