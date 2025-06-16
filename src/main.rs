use std::{collections::HashSet, env};

use anyhow::Result;
use axum::{
    extract::{Path, State}, routing::{get, post}, Json, Router
};
use dotenv::dotenv;
use serde::{Deserialize, Serialize};
use sqlx::{PgPool, postgres::PgPoolOptions, query, query_as, types::time::Date};
use time::format_description;
use uuid::Uuid;

#[derive(Clone)]
struct ApiContext {
    pub db: PgPool,
}

#[derive(sqlx::Type, Serialize, Deserialize, Debug)]
struct Album {
    id: Uuid,
    title: String,
    artists: Option<Vec<Artist>>,
    #[serde(with = "my_date_format")]
    date: Date,
    genres: Option<Vec<Genre>>,
    url: String,
}
#[derive(Serialize, Deserialize, Debug)]
struct NewAlbum {
    album: String,
    artists: Vec<String>,
    #[serde(with = "my_date_format")]
    date: Date,
    genres: Vec<String>,
    url: String,
}

#[derive(Serialize, Deserialize)]
struct InsertedAlbum {
    id: Uuid,
    title: String,
    #[serde(with = "my_date_format")]
    date: Date,
    url: String,
}

#[derive(sqlx::Type, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, Debug)]
struct Artist {
    id: Uuid,
    name: String,
}

#[derive(sqlx::Type, Serialize, Deserialize, Debug)]
struct Genre {
    id: Uuid,
    name: String,
}

mod my_date_format {
    use anyhow::Result;
    use serde::{self, Deserialize, Deserializer, Serializer};
    use sqlx::types::time::Date;
    use time::format_description;

    // The signature of a serialize_with function must follow the pattern:
    //
    //    fn serialize<S>(&T, S) -> Result<S::Ok, S::Error>
    //    where
    //        S: Serializer
    //
    // although it may also be generic over the input types T.
    pub fn serialize<S>(date: &Date, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let format = format_description::parse("[year]-[month]-[day]").unwrap();
        let s = date.format(&format).unwrap();
        serializer.serialize_str(&s)
    }

    // The signature of a deserialize_with function must follow the pattern:
    //
    //    fn deserialize<'de, D>(D) -> Result<T, D::Error>
    //    where
    //        D: Deserializer<'de>
    //
    // although it may also be generic over the output types T.
    pub fn deserialize<'de, D>(deserializer: D) -> Result<Date, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let format = format_description::parse("[year]-[month]-[day]").unwrap();
        let date = Date::parse(&s, &format).map_err(serde::de::Error::custom)?;
        Ok(date)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect(&env::var("DATABASE_URL")?)
        .await?;
    // build our application with a single route
    let router = Router::new()
        .route("/", get(get_albums))
        .route("/", post(add_albums))
        .route("/date/{date}", get(get_albums_for_date))
        .route("/genre/{genre}", get(get_albums_for_genre))
        .with_state(ApiContext { db });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5000").await.unwrap();
    axum::serve(listener, router).await.unwrap();

    Ok(())
}

async fn get_albums(State(state): State<ApiContext>,) -> Json<Vec<Album>> {
    let q: Vec<Album> = query_as!(
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
        GROUP BY al.id"#
    )
    .fetch_all(&state.db)
    .await
    .unwrap();
    Json(q)
}

async fn get_albums_for_genre(
    State(state): State<ApiContext>,
    Path(genre): Path<String>,
) -> Json<Vec<Album>> {
    let q: Vec<Album> = query_as!(
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
        GROUP BY al.id"#,
        genre
    )
    .fetch_all(&state.db)
    .await
    .unwrap();
    Json(q)
}

async fn get_albums_for_date(
    State(state): State<ApiContext>,
    Path(date): Path<String>,
) -> Json<Vec<Album>> {
    let format = format_description::parse("[year]-[month]-[day]").unwrap();
    let parsed = Date::parse(&date, &format).unwrap();
    let q: Vec<Album> = query_as!(
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
        parsed
    )
    .fetch_all(&state.db)
    .await
    .unwrap();
    Json(q)
}

async fn add_albums(
    State(state): State<ApiContext>,
    Json(payload): Json<Vec<NewAlbum>>,
) -> Json<Vec<Album>> {
    let stream = payload.iter().map(async |album| {
        let inserted_album = add_album(&album, &state.db).await;
        let genres = add_genres(&album.genres, &state.db).await;
        add_album_genres(&inserted_album, &genres, &state.db).await;
        let artists = add_artists(&album.artists, &state.db).await;
        add_album_artists(&inserted_album, &artists, &state.db).await;
        let actual_album = Album {
            id: inserted_album.id,
            title: inserted_album.title,
            artists: Some(artists),
            date: inserted_album.date,
            genres: Some(genres),
            url: inserted_album.url,
        };
        actual_album
    });
    let ret = Json(futures::future::join_all(stream).await);
    ret
}

async fn get_genre(genre: String, db: &PgPool) -> Genre {
    let _ = query_as!(
        Genre,
        "INSERT INTO genres(name) VALUES ($1) ON CONFLICT DO NOTHING RETURNING id, name",
        genre
    )
    .fetch_one(db)
    .await;
    let genre = query_as!(Genre, "SELECT * FROM genres WHERE name = $1", genre)
        .fetch_one(db)
        .await;
    return genre.unwrap();
}

async fn add_genres(genres: &Vec<String>, db: &PgPool) -> Vec<Genre> {
    let stream = genres
        .iter()
        .map(async |g| get_genre(g.to_string(), db).await);
    futures::future::join_all(stream).await
}

async fn get_artist(artist: String, db: &PgPool) -> Artist {
    let _ = query_as!(
        Artist,
        "INSERT INTO artists(name) VALUES ($1) ON CONFLICT DO NOTHING RETURNING id, name",
        artist
    )
    .fetch_one(db)
    .await;
    let artist = query_as!(Artist, "SELECT * FROM artists WHERE name = $1", artist)
        .fetch_one(db)
        .await;
    return artist.unwrap();
}

async fn add_artists(artists: &Vec<String>, db: &PgPool) -> Vec<Artist> {
    fn dedup(v: &mut Vec<Artist>) {
        let mut set = HashSet::new();

        v.retain(|x| set.insert(x.clone()));
    }
    let stream = artists
        .iter()
        .map(async |g| get_artist(g.to_string(), db).await);
    let mut db_artists = futures::future::join_all(stream).await;
    dedup(&mut db_artists);
    db_artists
}

async fn add_album(album: &NewAlbum, db: &PgPool) -> InsertedAlbum {
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
    .await
    .unwrap();
    return new_album.unwrap();
}

async fn add_album_genres(album: &InsertedAlbum, genres: &Vec<Genre>, db: &PgPool) {
    for genre in genres {
        let _ = query!(
            "INSERT INTO album_genres(album_id, genre_id) VALUES ($1, $2)",
            album.id,
            genre.id
        )
        .execute(db)
        .await;
    }
}

async fn add_album_artists(album: &InsertedAlbum, artists: &Vec<Artist>, db: &PgPool) {
    for artist in artists {
        let _ = query!(
            "INSERT INTO album_artists(album_id, artist_id) VALUES ($1, $2)",
            album.id,
            artist.id
        )
        .execute(db)
        .await;
    }
}
