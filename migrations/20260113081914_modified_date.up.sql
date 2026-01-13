-- Add up migration script here
ALTER TABLE albums
ADD COLUMN modified_date DATE NOT NULL DEFAULT NOW();