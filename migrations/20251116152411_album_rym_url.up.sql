-- Add up migration script here
ALTER TABLE albums
    ADD rym_url TEXT UNIQUE;