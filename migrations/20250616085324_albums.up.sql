-- Add up migration script here
CREATE TABLE albums(
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    title TEXT NOT NULL,
    date DATE NOT NULL,
    url TEXT NOT NULL,
    UNIQUE(url)
)