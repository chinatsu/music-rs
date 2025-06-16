-- Add up migration script here
CREATE TABLE genres(
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    name TEXT NOT NULL UNIQUE
);