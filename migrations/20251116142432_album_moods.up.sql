-- Add up migration script here
CREATE TABLE moods(
    id UUID PRIMARY KEY DEFAULT uuidv7(),
    name TEXT NOT NULL UNIQUE
);
CREATE TABLE album_moods(
    mood_id uuid REFERENCES moods(id) ON UPDATE CASCADE ON DELETE CASCADE,
    album_id uuid REFERENCES albums(id) ON UPDATE CASCADE,
    CONSTRAINT album_moods_pkey PRIMARY KEY (mood_id, album_id)
);