-- Add up migration script here
CREATE TABLE IF NOT EXISTS messages (
    id TEXT PRIMARY KEY,
    thread_name TEXT,
    role TEXT NOT NULL,
    content TEXT NOT NULL
);
