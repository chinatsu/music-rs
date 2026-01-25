-- Add down migration script here
ALTER TABLE tracks DROP COLUMN localized_title;
ALTER TABLE artists DROP COLUMN localized_name;