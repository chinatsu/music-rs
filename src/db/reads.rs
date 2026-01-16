use sqlx::{PgPool, Postgres, QueryBuilder, Row, query_as};
use uuid::Uuid;

use super::filters::{
    apply_date_range_filter, apply_genre_filter, apply_mood_filter, apply_pagination,
    apply_rating_filter,
};
use crate::{
    Result,
    types::{Album, Artist, Genre, Mood, SimilarGenre, SimilarMood, Track},
};

pub async fn get_albums(
    db: &PgPool,
    page: i64,
    limit: i64,
    filters: &crate::routes::AlbumFilter,
) -> Result<Vec<Album>> {
    let mut builder: QueryBuilder<Postgres> = QueryBuilder::new(
        r#"
        SELECT 
            al.id, al.title, al.date, al.url, al.rym_url, al.score, al.voters, al.modified_date,
            COALESCE((SELECT json_agg(DISTINCT jsonb_build_object('id', ar.id, 'name', ar.name))
                      FROM album_artists aa
                      JOIN artists ar ON aa.artist_id = ar.id
                      WHERE aa.album_id = al.id), '[]') as artists,
            COALESCE((SELECT json_agg(DISTINCT jsonb_build_object('id', g.id, 'name', g.name))
                      FROM album_genres ag
                      JOIN genres g ON ag.genre_id = g.id
                      WHERE ag.album_id = al.id), '[]') as genres,
            COALESCE((SELECT json_agg(DISTINCT jsonb_build_object('id', m.id, 'name', m.name))
                      FROM album_moods am
                      JOIN moods m ON am.mood_id = m.id
                      WHERE am.album_id = al.id), '[]') as moods,
            COALESCE((SELECT json_agg(jsonb_build_object('track_number', t.track_number, 'title', t.title) ORDER BY t.track_number)
                      FROM tracks t
                      WHERE t.album_id = al.id), '[]') as tracks
        FROM albums al
        WHERE al.voters != 0
        "#,
    );

    apply_genre_filter(&mut builder, &filters.genres);
    apply_mood_filter(&mut builder, &filters.moods);
    apply_rating_filter(&mut builder, filters.min_rating);
    apply_date_range_filter(&mut builder, filters.since, filters.to);
    apply_pagination(&mut builder, page, limit);

    let query = builder.build();
    Ok(query
        .fetch_all(db)
        .await?
        .into_iter()
        .map(|row| {
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
        })
        .collect())
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

pub async fn get_genre(db: &PgPool, genre_id: Uuid) -> Result<Genre> {
    let genre = query_as!(Genre, "SELECT * FROM genres WHERE id = $1", genre_id)
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
