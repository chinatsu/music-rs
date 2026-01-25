use sqlx::{PgPool, query_as};
use uuid::Uuid;

use crate::{
    Result,
    error::AppError,
    types::{Album, Artist, Genre, InsertedAlbum, Mood, NewAlbum, NewArtist, NewTrack, Track},
};

pub async fn register_album(db: &PgPool, album: &NewAlbum) -> Result<Album> {
    let inserted_album = add_album(album, db).await?;
    let genres = add_genres(&album.genres, db).await?;
    let artists = add_artists(&album.artists, db).await?;
    let moods = add_moods(&album.moods, db).await?;
    let tracks = add_tracks(inserted_album.id, &album.tracks, db).await?;

    add_album_artists(&inserted_album, &artists, db).await?;
    add_album_genres(&inserted_album, &genres, db).await?;
    add_album_moods(&inserted_album, &moods, db).await?;
    Ok(Album {
        id: inserted_album.id,
        title: inserted_album.title,
        localized_title: inserted_album.localized_title,
        artists: Some(artists),
        date: inserted_album.date,
        genres: Some(genres),
        moods: Some(moods),
        tracks: Some(tracks),
        url: inserted_album.url,
        rym_url: inserted_album.rym_url,
        score: inserted_album.score,
        voters: inserted_album.voters,
        modified_date: sqlx::types::chrono::Utc::now().date_naive(),
    })
}

async fn add_album(album: &NewAlbum, db: &PgPool) -> Result<InsertedAlbum> {
    let _: std::result::Result<InsertedAlbum, sqlx::Error> = query_as!(
        InsertedAlbum,
        "INSERT INTO albums(title, date, url, rym_url, score, voters, localized_title)
        VALUES ($1, $2, $3, $4, $5, $6, $7)
        ON CONFLICT (url) DO UPDATE
        SET title = $1,
            date = $2,
            url = $3,
            rym_url = $4,
            score = $5,
            voters = $6,
            localized_title = $7,
            modified_date = DEFAULT
        RETURNING id, title, date, url, rym_url, score, voters, localized_title, modified_date",
        album.album,
        album.date,
        album.url,
        album.rym_url,
        album.score,
        album.voters,
        album.localized_title
    )
    .fetch_one(db)
    .await;
    let new_album = query_as!(
        InsertedAlbum,
        "SELECT * FROM albums WHERE url = $1",
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
        sqlx::query!(
            "INSERT INTO album_artists(album_id, artist_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            album.id,
            artist.id
        )
        .execute(db)
        .await?;
    }
    Ok(())
}

async fn add_album_genres(album: &InsertedAlbum, genres: &Vec<Genre>, db: &PgPool) -> Result<()> {
    for genre in genres {
        sqlx::query!(
            "INSERT INTO album_genres(album_id, genre_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            album.id,
            genre.id
        )
        .execute(db)
        .await?;
    }
    Ok(())
}

async fn add_album_moods(album: &InsertedAlbum, moods: &Vec<Mood>, db: &PgPool) -> Result<()> {
    for mood in moods {
        sqlx::query!(
            "INSERT INTO album_moods(album_id, mood_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
            album.id,
            mood.id
        )
        .execute(db)
        .await?;
    }
    Ok(())
}

async fn add_genres(genres: &[String], db: &PgPool) -> Result<Vec<Genre>> {
    let stream = genres
        .iter()
        .map(async |g| get_or_create_genre(g.to_string(), db).await.unwrap());
    Ok(futures::future::join_all(stream).await)
}

async fn add_moods(moods: &[String], db: &PgPool) -> Result<Vec<Mood>> {
    let stream = moods
        .iter()
        .map(async |m| get_or_create_mood(m.to_string(), db).await.unwrap());
    Ok(futures::future::join_all(stream).await)
}

async fn add_artists(artists: &[NewArtist], db: &PgPool) -> Result<Vec<Artist>> {
    let stream = artists
        .iter()
        .map(async |a| get_or_create_artist(a.clone(), db).await.unwrap());
    Ok(futures::future::join_all(stream).await)
}

async fn add_tracks(album_id: Uuid, tracks: &[NewTrack], db: &PgPool) -> Result<Vec<Track>> {
    sqlx::query!("DELETE FROM tracks WHERE album_id = $1", album_id)
        .execute(db)
        .await?;
    for track in tracks {
        let artist_id = if let Some(artist) = &track.artist {
            Some(get_or_create_artist(artist.clone(), db).await?.id)
        } else {
            None
        };
        sqlx::query!(
            "INSERT INTO tracks(album_id, track_number, title, artist) 
            VALUES ($1, $2, $3, $4)",
            album_id,
            track.track_number,
            track.title,
            artist_id
        )
        .execute(db)
        .await?;
    }
    let returned_tracks = sqlx::query_as!(
        Track,
        r#"SELECT t.id, t.track_number, t.title, t.localized_title, a as "artist?: Artist"
        FROM tracks t
        LEFT JOIN artists a ON t.artist = a.id
        WHERE t.album_id = $1
        ORDER BY t.track_number"#,
        album_id
    )
    .fetch_all(db)
    .await?;
    Ok(returned_tracks)
}

async fn get_or_create_genre(genre: String, db: &PgPool) -> Result<Genre> {
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

async fn get_or_create_mood(mood: String, db: &PgPool) -> Result<Mood> {
    let _ = query_as!(
        Mood,
        "INSERT INTO moods(name) VALUES ($1) ON CONFLICT DO NOTHING RETURNING id, name",
        mood
    )
    .fetch_one(db)
    .await;
    let mood = query_as!(Mood, "SELECT * FROM moods WHERE name = $1", mood)
        .fetch_one(db)
        .await?;
    Ok(mood)
}

async fn get_or_create_artist(artist: NewArtist, db: &PgPool) -> Result<Artist> {
    let _ = query_as!(
        Artist,
        r#"INSERT INTO artists(name, localized_name) 
        VALUES ($1, $2)
        ON CONFLICT (name) 
        DO UPDATE SET localized_name = $2
        RETURNING id, name, localized_name"#,
        artist.name,
        artist.localized_name
    )
    .fetch_one(db)
    .await;
    let artist = query_as!(Artist, "SELECT * FROM artists WHERE name = $1", artist.name)
        .fetch_one(db)
        .await?;
    Ok(artist)
}
