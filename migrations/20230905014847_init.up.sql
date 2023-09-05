-- Add up migration script here
BEGIN;

    CREATE EXTENSION IF NOT EXISTS vector;
    
    CREATE TABLE IF NOT EXISTS threads (
        name TEXT NOT NULL PRIMARY KEY UNIQUE
    );

    CREATE TABLE IF NOT EXISTS files (
        id TEXT PRIMARY KEY,
        thread_name TEXT,
        filepath TEXT NOT NULL,
        parent_dir_path TEXT NOT NULL,
        summary TEXT NOT NULL,
        summary_embedding vector(384)
    );

    CREATE TABLE IF NOT EXISTS file_chunks (
        id TEXT PRIMARY KEY,
        parent_file_id TEXT NOT NULL,
        parent_filepath TEXT NOT NULL,
        idx smallint NOT NULL,
        content TEXT NOT NULL,
        content_embedding vector(384)
    );

    CREATE TABLE IF NOT EXISTS messages (
        id TEXT PRIMARY KEY,
        thread_name TEXT,
        role TEXT NOT NULL,
        content TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS io (
        id TEXT PRIMARY KEY,
        input TEXT,
        output TEXT
    );

COMMIT;
