-- Add up migration script here
-- Add up migration script here
CREATE TABLE album_genres(
    genre_id uuid REFERENCES genres(id) ON UPDATE CASCADE ON DELETE CASCADE,
    album_id uuid REFERENCES albums(id) ON UPDATE CASCADE,
    CONSTRAINT album_genres_pkey PRIMARY KEY (genre_id, album_id)
);