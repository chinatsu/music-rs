-- Add up migration script here
CREATE TABLE artists(
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    name TEXT NOT NULL UNIQUE
);