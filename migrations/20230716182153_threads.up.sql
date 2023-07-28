-- Add up migration script here
CREATE TABLE IF NOT EXISTS threads (
    name TEXT NOT NULL PRIMARY KEY UNIQUE
);
