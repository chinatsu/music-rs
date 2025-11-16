use sqlx::{PgPool, query, query_as};
use time::Date;
use uuid::Uuid;

use crate::{
    Result,
    error::AppError,
    types::{Album, Artist, Genre, InsertedAlbum, Mood, NewAlbum, SimilarGenre, SimilarMood},
};

pub async fn register_album(db: &PgPool, album: &NewAlbum) -> Result<Album> {
    let inserted_album = add_album(album, db).await?;
    let genres = add_genres(&album.genres, db).await?;
    let artists = add_artists(&album.artists, db).await?;
    let moods = add_moods(&album.moods, db).await?;

    add_album_artists(&inserted_album, &artists, db).await?;
    add_album_genres(&inserted_album, &genres, db).await?;
    add_album_moods(&inserted_album, &moods, db).await?;
    Ok(Album {
        id: inserted_album.id,
        title: inserted_album.title,
        artists: Some(artists),
        date: inserted_album.date,
        genres: Some(genres),
        moods: Some(moods),
        url: inserted_album.url,
        score: inserted_album.score,
        voters: inserted_album.voters,
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
            al.score as "score",
            al.voters as "voters",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (ar.id, ar.name)) filter (where ar.id is not null), '{NULL}'), '{}') as "artists?: Vec<Artist>",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (g.id, g.name)) filter (where g.id is not null), '{NULL}'), '{}') as "genres?: Vec<Genre>",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (m.id, m.name)) filter (where m.id is not null), '{NULL}'), '{}') as "moods?: Vec<Mood>"
        FROM albums al
        LEFT JOIN album_artists aa ON al.id = aa.album_id
        LEFT JOIN artists ar ON aa.artist_id = ar.id
        LEFT JOIN album_genres ag ON al.id = ag.album_id
        LEFT JOIN genres g ON ag.genre_id = g.id
        LEFT JOIN album_moods am ON al.id = am.album_id
        LEFT JOIN moods m ON am.mood_id = m.id
        WHERE al.voters != 0
        GROUP BY al.id
        ORDER BY al.date desc, al.score desc"#
    )
    .fetch_all(db)
    .await?)
}

pub async fn get_albums_for_genre(db: &PgPool, genre_id: Uuid) -> Result<Vec<Album>> {
    Ok(query_as!(
        Album,
        r#"
        SELECT 
            al.id as "id",
            al.title as "title",
            al.date as "date",
            al.url as "url",
            al.score as "score",
            al.voters as "voters",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (ar.id, ar.name)) filter (where ar.id is not null), '{NULL}'), '{}') as "artists?: Vec<Artist>",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (g.id, g.name)) filter (where g.id is not null), '{NULL}'), '{}') as "genres?: Vec<Genre>",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (m.id, m.name)) filter (where m.id is not null), '{NULL}'), '{}') as "moods?: Vec<Mood>"
        FROM albums al
        LEFT JOIN album_artists aa ON al.id = aa.album_id
        LEFT JOIN artists ar ON aa.artist_id = ar.id
        LEFT JOIN album_genres ag ON al.id = ag.album_id
        LEFT JOIN genres g ON ag.genre_id = g.id
        LEFT JOIN album_moods am ON al.id = am.album_id
        LEFT JOIN moods m ON am.mood_id = m.id
        WHERE $1 = ANY(SELECT ge.id FROM genres ge JOIN album_genres alg ON alg.genre_id = ge.id AND al.id = alg.album_id) AND al.voters != 0
        GROUP BY al.id
        ORDER BY al.date desc, al.score desc"#,
        genre_id
    )
    .fetch_all(db)
    .await?)
}


pub async fn get_albums_for_mood(db: &PgPool, mood_id: Uuid) -> Result<Vec<Album>> {
    Ok(query_as!(
        Album,
        r#"
        SELECT 
            al.id as "id",
            al.title as "title",
            al.date as "date",
            al.url as "url",
            al.score as "score",
            al.voters as "voters",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (ar.id, ar.name)) filter (where ar.id is not null), '{NULL}'), '{}') as "artists?: Vec<Artist>",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (g.id, g.name)) filter (where g.id is not null), '{NULL}'), '{}') as "genres?: Vec<Genre>",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (m.id, m.name)) filter (where m.id is not null), '{NULL}'), '{}') as "moods?: Vec<Mood>"
        FROM albums al
        LEFT JOIN album_artists aa ON al.id = aa.album_id
        LEFT JOIN artists ar ON aa.artist_id = ar.id
        LEFT JOIN album_genres ag ON al.id = ag.album_id
        LEFT JOIN genres g ON ag.genre_id = g.id
        LEFT JOIN album_moods am ON al.id = am.album_id
        LEFT JOIN moods m ON am.mood_id = m.id
        WHERE $1 = ANY(SELECT mo.id FROM moods mo JOIN album_moods alm ON alm.mood_id = mo.id AND al.id = alm.album_id) AND al.voters != 0
        GROUP BY al.id
        ORDER BY al.date desc, al.score desc"#,
        mood_id
    )
    .fetch_all(db)
    .await?)
}

pub async fn get_albums_for_artist(db: &PgPool, artist_id: Uuid) -> Result<Vec<Album>> {
    Ok(query_as!(
        Album,
        r#"
        SELECT 
            al.id as "id",
            al.title as "title",
            al.date as "date",
            al.url as "url",
            al.score as "score",
            al.voters as "voters",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (ar.id, ar.name)) filter (where ar.id is not null), '{NULL}'), '{}') as "artists?: Vec<Artist>",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (g.id, g.name)) filter (where g.id is not null), '{NULL}'), '{}') as "genres?: Vec<Genre>",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (m.id, m.name)) filter (where m.id is not null), '{NULL}'), '{}') as "moods?: Vec<Mood>"
        FROM albums al
        LEFT JOIN album_artists aa ON al.id = aa.album_id
        LEFT JOIN artists ar ON aa.artist_id = ar.id
        LEFT JOIN album_genres ag ON al.id = ag.album_id
        LEFT JOIN genres g ON ag.genre_id = g.id
        LEFT JOIN album_moods am ON al.id = am.album_id
        LEFT JOIN moods m ON am.mood_id = m.id
        WHERE $1 = ANY(SELECT ar.id FROM artists ar JOIN album_artists ala ON ala.artist_id = ar.id AND al.id = ala.album_id) AND al.voters != 0
        GROUP BY al.id
        ORDER BY al.date desc, al.score desc"#,
        artist_id
    ).fetch_all(db)
    .await?
    )
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
            al.score as "score",
            al.voters as "voters",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (ar.id, ar.name)) filter (where ar.id is not null), '{NULL}'), '{}') as "artists?: Vec<Artist>",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (g.id, g.name)) filter (where g.id is not null), '{NULL}'), '{}') as "genres?: Vec<Genre>",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (m.id, m.name)) filter (where m.id is not null), '{NULL}'), '{}') as "moods?: Vec<Mood>"
        FROM albums al
        LEFT JOIN album_artists aa ON al.id = aa.album_id
        LEFT JOIN artists ar ON aa.artist_id = ar.id
        LEFT JOIN album_genres ag ON al.id = ag.album_id
        LEFT JOIN genres g ON ag.genre_id = g.id
        LEFT JOIN album_moods am ON al.id = am.album_id
        LEFT JOIN moods m ON am.mood_id = m.id
        WHERE al.date = $1 AND al.voters != 0
        GROUP BY al.id
        ORDER BY al.score desc"#,
        date
    )
    .fetch_all(db)
    .await?)
}

pub async fn get_similar_genres(db: &PgPool, genre_id: Uuid) -> Result<Vec<SimilarGenre>> {
    let album_genres: Vec<SimilarGenre> = query_as!(
        SimilarGenre,
        r#"SELECT
            related_genre_details.id AS id,
            related_genre_details.name AS name,
            COUNT(1) AS count
        FROM album_genres AS related_albums
        INNER JOIN album_genres AS related_genres
            ON related_albums.album_id = related_genres.album_id
        INNER JOIN genres AS related_genre_details
            ON related_genres.genre_id = related_genre_details.id
        WHERE related_albums.genre_id = $1
        GROUP BY related_genre_details.id
        ORDER BY count DESC
        "#,
        genre_id
    )
    .fetch_all(db)
    .await?;
    Ok(album_genres)
}

pub async fn get_similar_moods(db: &PgPool, mood_id: Uuid) -> Result<Vec<SimilarMood>> {
    let album_moods: Vec<SimilarMood> = query_as!(
        SimilarMood,
        r#"SELECT
            related_mood_details.id AS id,
            related_mood_details.name AS name,
            COUNT(1) AS count
        FROM album_moods AS related_albums
        INNER JOIN album_moods AS related_moods
            ON related_albums.album_id = related_moods.album_id
        INNER JOIN moods AS related_mood_details
            ON related_moods.mood_id = related_mood_details.id
        WHERE related_albums.mood_id = $1
        GROUP BY related_mood_details.id
        ORDER BY count DESC
        "#,
        mood_id
    )
    .fetch_all(db)
    .await?;
    Ok(album_moods)
}

pub async fn get_genre(db: &PgPool, genre_id: Uuid) -> Result<Genre> {
    let genre = query_as!(Genre, "SELECT * FROM genres WHERE id = $1", genre_id)
        .fetch_one(db)
        .await?;
    Ok(genre)
}

async fn get_new_genre(genre: String, db: &PgPool) -> Result<Genre> {
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

pub async fn get_mood(db: &PgPool, mood_id: Uuid) -> Result<Mood> {
    let mood = query_as!(Mood, "SELECT * FROM moods WHERE id = $1", mood_id)
        .fetch_one(db)
        .await?;
    Ok(mood)
}

async fn get_new_mood(mood: String, db: &PgPool) -> Result<Mood> {
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
        "INSERT INTO albums(title, date, url, score, voters)
        VALUES ($1, $2, $3, $4, $5)
        ON CONFLICT (url) DO UPDATE
        SET title = $1,
            date = $2,
            url = $3,
            score = $4,
            voters = $5
        RETURNING id, title, date, url, score, voters",
        album.album,
        album.date,
        album.url,
        album.score,
        album.voters
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
        query!(
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
        query!(
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
        query!(
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
        .map(async |g| get_new_genre(g.to_string(), db).await.unwrap());
    Ok(futures::future::join_all(stream).await)
}

async fn add_moods(moods: &[String], db: &PgPool) -> Result<Vec<Mood>> {
    let stream = moods
        .iter()
        .map(async |m| get_new_mood(m.to_string(), db).await.unwrap());
    Ok(futures::future::join_all(stream).await)
}

async fn add_artists(artists: &[String], db: &PgPool) -> Result<Vec<Artist>> {
    let stream = artists
        .iter()
        .map(async |a| get_artist(a.to_string(), db).await.unwrap());
    Ok(futures::future::join_all(stream).await)
}
