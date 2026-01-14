use chrono::NaiveDate;
use sqlx::{PgPool, query, query_as, types::chrono::Utc, QueryBuilder, Postgres, Row};
use uuid::Uuid;

use crate::{
    Result,
    error::AppError,
    types::{
        Album, Artist, Genre, InsertedAlbum, Mood, NewAlbum, SimilarGenre, SimilarMood, Track,
    },
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
        artists: Some(artists),
        date: inserted_album.date,
        genres: Some(genres),
        moods: Some(moods),
        tracks: Some(tracks),
        url: inserted_album.url,
        rym_url: inserted_album.rym_url,
        score: inserted_album.score,
        voters: inserted_album.voters,
        modified_date: Utc::now().date_naive(),
    })
}

pub async fn get_albums(db: &PgPool, page: i64, limit: i64, filters: &crate::routes::AlbumFilter) -> Result<Vec<Album>> {
    // Parse genre and mood names if provided
    let genre_names: Option<Vec<String>> = filters.genres.as_ref().map(|g: &String| {
        g.split(',').map(|s: &str| s.trim().to_string()).collect()
    });
    let mood_names: Option<Vec<String>> = filters.moods.as_ref().map(|m: &String| {
        m.split(',').map(|s: &str| s.trim().to_string()).collect()
    });

    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        SELECT 
            al.id, al.title, al.date, al.url, al.rym_url, al.score, al.voters, al.modified_date,
            COALESCE(json_agg(DISTINCT jsonb_build_object('id', ar.id, 'name', ar.name)) FILTER (WHERE ar.id IS NOT NULL), '[]') as artists,
            COALESCE(json_agg(DISTINCT jsonb_build_object('id', g.id, 'name', g.name)) FILTER (WHERE g.id IS NOT NULL), '[]') as genres,
            COALESCE(json_agg(DISTINCT jsonb_build_object('id', m.id, 'name', m.name)) FILTER (WHERE m.id IS NOT NULL), '[]') as moods,
            COALESCE(json_agg(jsonb_build_object('track_number', t.track_number, 'title', t.title) ORDER BY t.track_number) FILTER (WHERE t.id IS NOT NULL), '[]') as tracks
        FROM albums al
        LEFT JOIN album_artists aa ON al.id = aa.album_id
        LEFT JOIN artists ar ON aa.artist_id = ar.id
        LEFT JOIN album_genres ag ON al.id = ag.album_id
        LEFT JOIN genres g ON ag.genre_id = g.id
        LEFT JOIN album_moods am ON al.id = am.album_id
        LEFT JOIN moods m ON am.mood_id = m.id
        LEFT JOIN tracks t ON al.id = t.album_id
        WHERE al.voters != 0
        "#
    );

    // Add genre filter
    if let Some(ref names) = genre_names {
        builder.push(" AND EXISTS (SELECT 1 FROM album_genres ag2 JOIN genres g2 ON ag2.genre_id = g2.id WHERE ag2.album_id = al.id AND g2.name = ANY(");
        builder.push_bind(names.as_slice());
        builder.push("))");
    }

    // Add mood filter
    if let Some(ref names) = mood_names {
        builder.push(" AND EXISTS (SELECT 1 FROM album_moods am2 JOIN moods m2 ON am2.mood_id = m2.id WHERE am2.album_id = al.id AND m2.name = ANY(");
        builder.push_bind(names.as_slice());
        builder.push("))");
    }

    // Add rating filter
    if let Some(min_rating) = filters.min_rating {
        builder.push(" AND al.score >= ");
        builder.push_bind(min_rating as f32);
    }

    // Add date range filters
    if let Some(since) = filters.since {
        builder.push(" AND al.date >= ");
        builder.push_bind(since);
    }
    if let Some(to) = filters.to {
        builder.push(" AND al.date <= ");
        builder.push_bind(to);
    }

    builder.push(" GROUP BY al.id ORDER BY al.date desc, al.score desc LIMIT ");
    builder.push_bind(limit);
    builder.push(" OFFSET ");
    builder.push_bind((page - 1) * limit);

    let query = builder.build();
    Ok(query.fetch_all(db).await?.into_iter().map(|row| {
        use sqlx::types::JsonValue;
        
        let artists_json: JsonValue = row.get("artists");
        let genres_json: JsonValue = row.get("genres");
        let moods_json: JsonValue = row.get("moods");
        let tracks_json: JsonValue = row.get("tracks");
        
        Album {
            id: row.get("id"),
            title: row.get("title"),
            date: row.get("date"),
            url: row.get("url"),
            rym_url: row.get("rym_url"),
            score: row.get("score"),
            voters: row.get("voters"),
            modified_date: row.get("modified_date"),
            artists: serde_json::from_value(artists_json).ok(),
            genres: serde_json::from_value(genres_json).ok(),
            moods: serde_json::from_value(moods_json).ok(),
            tracks: serde_json::from_value(tracks_json).ok(),
        }
    }).collect())
}

pub async fn get_albums_for_genre(
    db: &PgPool,
    genre_id: Uuid,
    page: i64,
    limit: i64,
) -> Result<Vec<Album>> {
    Ok(query_as!(
        Album,
        r#"
        SELECT 
            al.id as "id",
            al.title as "title",
            al.date as "date",
            al.url as "url",
            al.rym_url as "rym_url",
            al.score as "score",
            al.voters as "voters",
            al.modified_date as "modified_date",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (ar.id, ar.name)) filter (where ar.id is not null), '{NULL}'), '{}') as "artists?: Vec<Artist>",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (g.id, g.name)) filter (where g.id is not null), '{NULL}'), '{}') as "genres?: Vec<Genre>",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (m.id, m.name)) filter (where m.id is not null), '{NULL}'), '{}') as "moods?: Vec<Mood>",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (t.track_number, t.title)) filter (where t.id is not null), '{NULL}'), '{}') as "tracks?: Vec<Track>"
        FROM albums al
        LEFT JOIN album_artists aa ON al.id = aa.album_id
        LEFT JOIN artists ar ON aa.artist_id = ar.id
        LEFT JOIN album_genres ag ON al.id = ag.album_id
        LEFT JOIN genres g ON ag.genre_id = g.id
        LEFT JOIN album_moods am ON al.id = am.album_id
        LEFT JOIN moods m ON am.mood_id = m.id
        LEFT JOIN tracks t ON al.id = t.album_id
        WHERE $1 = ANY(SELECT ge.id FROM genres ge JOIN album_genres alg ON alg.genre_id = ge.id AND al.id = alg.album_id) AND al.voters != 0
        GROUP BY al.id
        ORDER BY al.date desc, al.score desc
        LIMIT $2
        OFFSET $3"#,
        genre_id,
        limit,
        (page - 1) * limit
    )
    .fetch_all(db)
    .await?)
}

pub async fn get_albums_for_mood(
    db: &PgPool,
    mood_id: Uuid,
    page: i64,
    limit: i64,
) -> Result<Vec<Album>> {
    Ok(query_as!(
        Album,
        r#"
        SELECT 
            al.id as "id",
            al.title as "title",
            al.date as "date",
            al.url as "url",
            al.rym_url as "rym_url",
            al.score as "score",
            al.voters as "voters",
            al.modified_date as "modified_date",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (ar.id, ar.name)) filter (where ar.id is not null), '{NULL}'), '{}') as "artists?: Vec<Artist>",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (g.id, g.name)) filter (where g.id is not null), '{NULL}'), '{}') as "genres?: Vec<Genre>",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (m.id, m.name)) filter (where m.id is not null), '{NULL}'), '{}') as "moods?: Vec<Mood>",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (t.track_number, t.title)) filter (where t.id is not null), '{NULL}'), '{}') as "tracks?: Vec<Track>"
        FROM albums al
        LEFT JOIN album_artists aa ON al.id = aa.album_id
        LEFT JOIN artists ar ON aa.artist_id = ar.id
        LEFT JOIN album_genres ag ON al.id = ag.album_id
        LEFT JOIN genres g ON ag.genre_id = g.id
        LEFT JOIN album_moods am ON al.id = am.album_id
        LEFT JOIN moods m ON am.mood_id = m.id
        LEFT JOIN tracks t ON al.id = t.album_id
        WHERE $1 = ANY(SELECT mo.id FROM moods mo JOIN album_moods alm ON alm.mood_id = mo.id AND al.id = alm.album_id) AND al.voters != 0
        GROUP BY al.id
        ORDER BY al.date desc, al.score desc
        LIMIT $2
        OFFSET $3"#,
        mood_id,
        limit,
        (page - 1) * limit
    )
    .fetch_all(db)
    .await?)
}

pub async fn get_albums_for_artist(
    db: &PgPool,
    artist_id: Uuid,
    page: i64,
    limit: i64,
) -> Result<Vec<Album>> {
    Ok(query_as!(
        Album,
        r#"
        SELECT 
            al.id as "id",
            al.title as "title",
            al.date as "date",
            al.url as "url",
            al.rym_url as "rym_url",
            al.score as "score",
            al.voters as "voters",
            al.modified_date as "modified_date",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (ar.id, ar.name)) filter (where ar.id is not null), '{NULL}'), '{}') as "artists?: Vec<Artist>",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (g.id, g.name)) filter (where g.id is not null), '{NULL}'), '{}') as "genres?: Vec<Genre>",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (m.id, m.name)) filter (where m.id is not null), '{NULL}'), '{}') as "moods?: Vec<Mood>",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (t.track_number, t.title)) filter (where t.id is not null), '{NULL}'), '{}') as "tracks?: Vec<Track>"
        FROM albums al
        LEFT JOIN album_artists aa ON al.id = aa.album_id
        LEFT JOIN artists ar ON aa.artist_id = ar.id
        LEFT JOIN album_genres ag ON al.id = ag.album_id
        LEFT JOIN genres g ON ag.genre_id = g.id
        LEFT JOIN album_moods am ON al.id = am.album_id
        LEFT JOIN moods m ON am.mood_id = m.id
        LEFT JOIN tracks t ON al.id = t.album_id
        WHERE $1 = ANY(SELECT ar.id FROM artists ar JOIN album_artists ala ON ala.artist_id = ar.id AND al.id = ala.album_id) AND al.voters != 0
        GROUP BY al.id
        ORDER BY al.date desc, al.score desc
        LIMIT $2
        OFFSET $3"#,
        artist_id,
        limit,
        (page - 1) * limit
    ).fetch_all(db)
    .await?
    )
}

pub async fn get_albums_for_date(
    db: &PgPool,
    date: NaiveDate,
    page: i64,
    limit: i64,
) -> Result<Vec<Album>> {
    Ok(query_as!(
        Album,
        r#"
        SELECT 
            al.id as "id",
            al.title as "title",
            al.date as "date",
            al.url as "url",
            al.rym_url as "rym_url",
            al.score as "score",
            al.voters as "voters",
            al.modified_date as "modified_date",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (ar.id, ar.name)) filter (where ar.id is not null), '{NULL}'), '{}') as "artists?: Vec<Artist>",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (g.id, g.name)) filter (where g.id is not null), '{NULL}'), '{}') as "genres?: Vec<Genre>",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (m.id, m.name)) filter (where m.id is not null), '{NULL}'), '{}') as "moods?: Vec<Mood>",
            COALESCE(NULLIF(ARRAY_AGG(DISTINCT (t.track_number, t.title)) filter (where t.id is not null), '{NULL}'), '{}') as "tracks?: Vec<Track>"
        FROM albums al
        LEFT JOIN album_artists aa ON al.id = aa.album_id
        LEFT JOIN artists ar ON aa.artist_id = ar.id
        LEFT JOIN album_genres ag ON al.id = ag.album_id
        LEFT JOIN genres g ON ag.genre_id = g.id
        LEFT JOIN album_moods am ON al.id = am.album_id
        LEFT JOIN moods m ON am.mood_id = m.id
        LEFT JOIN tracks t ON al.id = t.album_id
        WHERE al.date = $1 AND al.voters != 0
        GROUP BY al.id
        ORDER BY al.score desc
        LIMIT $2
        OFFSET $3"#,
        date,
        limit,
        (page - 1) * limit
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
        "INSERT INTO albums(title, date, url, rym_url, score, voters)
        VALUES ($1, $2, $3, $4, $5, $6)
        ON CONFLICT (url) DO UPDATE
        SET title = $1,
            date = $2,
            url = $3,
            rym_url = $4,
            score = $5,
            voters = $6,
            modified_date = DEFAULT
        RETURNING id, title, date, url, rym_url, score, voters, modified_date",
        album.album,
        album.date,
        album.url,
        album.rym_url,
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

async fn add_tracks(album_id: Uuid, tracks: &[Track], db: &PgPool) -> Result<Vec<Track>> {
    query!("DELETE FROM tracks WHERE album_id = $1", album_id)
        .execute(db)
        .await?;
    for track in tracks {
        query!(
            "INSERT INTO tracks(album_id, track_number, title) 
            VALUES ($1, $2, $3)",
            album_id,
            track.track_number,
            track.title
        )
        .execute(db)
        .await?;
    }
    Ok(tracks.to_vec())
}
