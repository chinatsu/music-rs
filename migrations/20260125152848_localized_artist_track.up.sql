-- Add up migration script here
ALTER TABLE artists ADD COLUMN localized_name TEXT;
ALTER TABLE tracks ADD COLUMN localized_title TEXT;