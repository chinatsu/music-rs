-- Add up migration script here
CREATE TABLE album_artists(
    artist_id uuid REFERENCES artists(id) ON UPDATE CASCADE ON DELETE CASCADE,
    album_id uuid REFERENCES albums(id) ON UPDATE CASCADE,
    CONSTRAINT album_artists_pkey PRIMARY KEY (artist_id, album_id)
);