use sqlx::{PgPool, query, query_as};
use time::Date;

use crate::{
    Result,
    error::AppError,
    types::{Album, Artist, Genre, InsertedAlbum, NewAlbum},
};

pub async fn register_album(db: &PgPool, album: &NewAlbum) -> Result<Album> {
    let inserted_album = add_album(album, db).await?;
    let genres = add_genres(&album.genres, db).await?;
    let artists = add_artists(&album.artists, db).await?;

    add_album_artists(&inserted_album, &artists, db).await?;
    add_album_genres(&inserted_album, &genres, db).await?;
    Ok(Album {
        id: inserted_album.id,
        title: inserted_album.title,
        artists: Some(artists),
        date: inserted_album.date,
        genres: Some(genres),
        url: inserted_album.url,
    })
}

pub async fn get_albums(db: &PgPool) -> Result<Vec<Album>> {
    Ok(query_as!(
        Album,
        r#"
        SELECT 
            al.id as "id",
            al.title as "title",
            al.date as "date",
            al.url as "url",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (ar.id, ar.name)) filter (where ar.id is not null), '{NULL}'), '{}') as "artists?: Vec<Artist>",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (g.id, g.name)) filter (where g.id is not null), '{NULL}'), '{}') as "genres?: Vec<Genre>"
        FROM albums al
        LEFT JOIN album_artists aa ON al.id = aa.album_id
        LEFT JOIN artists ar ON aa.artist_id = ar.id
        LEFT JOIN album_genres ag ON al.id = ag.album_id
        LEFT JOIN genres g ON ag.genre_id = g.id
        GROUP BY al.id
        ORDER BY al.date desc"#
    )
    .fetch_all(db)
    .await?)
}

pub async fn get_albums_for_genre(db: &PgPool, genre: String) -> Result<Vec<Album>> {
    Ok(query_as!(
        Album,
        r#"
        SELECT 
            al.id as "id",
            al.title as "title",
            al.date as "date",
            al.url as "url",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (ar.id, ar.name)) filter (where ar.id is not null), '{NULL}'), '{}') as "artists?: Vec<Artist>",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (g.id, g.name)) filter (where g.id is not null), '{NULL}'), '{}') as "genres?: Vec<Genre>"
        FROM albums al
        LEFT JOIN album_artists aa ON al.id = aa.album_id
        LEFT JOIN artists ar ON aa.artist_id = ar.id
        LEFT JOIN album_genres ag ON al.id = ag.album_id
        LEFT JOIN genres g ON ag.genre_id = g.id
        WHERE $1 = ANY(SELECT ge.name FROM genres ge JOIN album_genres alg ON alg.genre_id = ge.id AND al.id = alg.album_id)
        GROUP BY al.id
        ORDER BY al.date desc"#,
        genre
    )
    .fetch_all(db)
    .await?)
}

pub async fn get_albums_for_date(db: &PgPool, date: Date) -> Result<Vec<Album>> {
    Ok(query_as!(
        Album,
        r#"
        SELECT 
            al.id as "id",
            al.title as "title",
            al.date as "date",
            al.url as "url",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (ar.id, ar.name)) filter (where ar.id is not null), '{NULL}'), '{}') as "artists?: Vec<Artist>",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (g.id, g.name)) filter (where g.id is not null), '{NULL}'), '{}') as "genres?: Vec<Genre>"
        FROM albums al
        LEFT JOIN album_artists aa ON al.id = aa.album_id
        LEFT JOIN artists ar ON aa.artist_id = ar.id
        LEFT JOIN album_genres ag ON al.id = ag.album_id
        LEFT JOIN genres g ON ag.genre_id = g.id
        WHERE al.date = $1
        GROUP BY al.id"#,
        date
    )
    .fetch_all(db)
    .await?)
}

async fn get_genre(genre: String, db: &PgPool) -> Result<Genre> {
    let _ = query_as!(
        Genre,
        "INSERT INTO genres(name) VALUES ($1) ON CONFLICT DO NOTHING RETURNING id, name",
        genre
    )
    .fetch_one(db)
    .await;
    let genre = query_as!(Genre, "SELECT * FROM genres WHERE name = $1", genre)
        .fetch_one(db)
        .await?;
    Ok(genre)
}

async fn get_artist(artist: String, db: &PgPool) -> Result<Artist> {
    let _ = query_as!(
        Artist,
        "INSERT INTO artists(name) VALUES ($1) ON CONFLICT DO NOTHING RETURNING id, name",
        artist
    )
    .fetch_one(db)
    .await;
    let artist = query_as!(Artist, "SELECT * FROM artists WHERE name = $1", artist)
        .fetch_one(db)
        .await?;
    Ok(artist)
}

async fn add_album(album: &NewAlbum, db: &PgPool) -> Result<InsertedAlbum> {
    let _: std::result::Result<InsertedAlbum, sqlx::Error> = query_as!(
        InsertedAlbum,
        "INSERT INTO albums(title, date, url) VALUES ($1, $2, $3) ON CONFLICT DO NOTHING RETURNING id, title, date, url",
        album.album,
        album.date,
        album.url
    )
    .fetch_one(db)
    .await;
    let new_album = query_as!(
        InsertedAlbum,
        "SELECT * FROM albums WHERE title = $1 AND date = $2 AND url = $3",
        album.album,
        album.date,
        album.url
    )
    .fetch_optional(db)
    .await?;
    if let Some(album) = new_album {
        return Ok(album);
    }
    Err(AppError::Sqlx(sqlx::Error::ColumnNotFound(
        "Couldn't find the row we just inserted".into(),
    )))
}

async fn add_album_artists(
    album: &InsertedAlbum,
    artists: &Vec<Artist>,
    db: &PgPool,
) -> Result<()> {
    for artist in artists {
        query!(
            "INSERT INTO album_artists(album_id, artist_id) VALUES ($1, $2)",
            album.id,
            artist.id
        )
        .execute(db)
        .await?;
    }
    Ok(())
}

async fn add_album_genres(
    album: &InsertedAlbum,
    genres: &Vec<Genre>,
    db: &PgPool,
) -> Result<()> {
    for genre in genres {
        query!(
            "INSERT INTO album_genres(album_id, genre_id) VALUES ($1, $2)",
            album.id,
            genre.id
        )
        .execute(db)
        .await?;
    }
    Ok(())
}

async fn add_genres(genres: &[String], db: &PgPool) -> Result<Vec<Genre>> {
    let stream = genres
        .iter()
        .map(async |g| get_genre(g.to_string(), db).await.unwrap());
    Ok(futures::future::join_all(stream).await)
}

async fn add_artists(artists: &[String], db: &PgPool) -> Result<Vec<Artist>> {
    let stream = artists
        .iter()
        .map(async |a| get_artist(a.to_string(), db).await.unwrap());
    Ok(futures::future::join_all(stream).await)
}
