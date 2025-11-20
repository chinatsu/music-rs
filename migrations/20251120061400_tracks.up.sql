-- Add up migration script here
CREATE TABLE tracks (
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    track_number INT NOT NULL,
    title TEXT NOT NULL,
    album_id UUID REFERENCES albums(id) ON DELETE CASCADE,
    UNIQUE(album_id, track_number)
);