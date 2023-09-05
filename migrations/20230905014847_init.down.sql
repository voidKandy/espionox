-- Add down migration script here
BEGIN;
    DROP EXTENSION vector;
    DROP TABLE IF EXISTS threads;
    DROP TABLE IF EXISTS files;
    DROP TABLE IF EXISTS file_chunks;
    DROP TABLE IF EXISTS messages;
    DROP TABLE IF EXISTS io;
COMMIT;
