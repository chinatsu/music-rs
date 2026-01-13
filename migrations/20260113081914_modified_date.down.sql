-- Add down migration script here
ALTER TABLE albums
DROP COLUMN modified_date;