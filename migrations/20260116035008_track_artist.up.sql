-- Add up migration script here
ALTER TABLE tracks ADD COLUMN artist UUID REFERENCES artists(id);