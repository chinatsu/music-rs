-- Add down migration script here

ALTER TABLE albums
    DROP COLUMN rym_url;