use axum::{
    Json,
    extract::{Path, Query, State},
};
use chrono::NaiveDate;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    ApiContext, Result, db,
    types::{Album, GenreInfo, MoodInfo, NewAlbum},
};

#[derive(Deserialize, Clone)]
pub struct AlbumFilter {
    #[serde(default)]
    pub page: i64,
    #[serde(default)]
    pub limit: i64,

    #[serde(default, deserialize_with = "deserialize_comma_separated")]
    pub genres: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_comma_separated")]
    pub moods: Vec<String>,
    pub min_rating: Option<f64>,
    pub since: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
}

fn deserialize_comma_separated<'de, D>(deserializer: D) -> std::result::Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    Ok(
        s.map(|s| s.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_default(),
    )
}

pub async fn add_albums(
    State(state): State<ApiContext>,
    Json(payload): Json<Vec<NewAlbum>>,
) -> Result<Json<Vec<Album>>> {
    let stream = payload
        .iter()
        .map(async |album| Ok(db::register_album(&state.db, album).await?));
    let albums: Result<Vec<Album>> = futures::future::try_join_all(stream).await;
    Ok(Json(albums?))
}

pub async fn get_albums(
    State(state): State<ApiContext>,
    album_filter: Query<AlbumFilter>,
) -> Result<Json<Vec<Album>>> {
    let (page, limit) = get_pagination_params(album_filter.clone());
    Ok(Json(
        db::get_albums(&state.db, page, limit, &album_filter).await?,
    ))
}

pub async fn get_genre(
    State(state): State<ApiContext>,
    Path(genre): Path<String>,
    album_filter: Query<AlbumFilter>,
) -> Result<Json<GenreInfo>> {
    let genre_id = Uuid::parse_str(&genre)?;
    let (page, limit) = get_pagination_params(album_filter);
    let db_genre = db::get_genre(&state.db, genre_id).await?;
    let db_similar_genres = db::get_similar_genres(&state.db, genre_id).await?;
    let db_genre_albums = db::get_albums_for_genre(&state.db, genre_id, page, limit).await?;

    Ok(Json(GenreInfo {
        genre: db_genre,
        similar_genres: db_similar_genres,
        albums: db_genre_albums,
    }))
}

pub async fn get_mood(
    State(state): State<ApiContext>,
    Path(mood): Path<String>,
    album_filter: Query<AlbumFilter>,
) -> Result<Json<MoodInfo>> {
    let mood_id = Uuid::parse_str(&mood)?;
    let (page, limit) = get_pagination_params(album_filter);
    let db_mood = db::get_mood(&state.db, mood_id).await?;
    let db_similar_moods = db::get_similar_moods(&state.db, mood_id).await?;
    let db_mood_albums = db::get_albums_for_mood(&state.db, mood_id, page, limit).await?;
    Ok(Json(MoodInfo {
        mood: db_mood,
        similar_moods: db_similar_moods,
        albums: db_mood_albums,
    }))
}

pub async fn get_artist(
    State(state): State<ApiContext>,
    Path(artist_id): Path<String>,
    album_filter: Query<AlbumFilter>,
) -> Result<Json<Vec<Album>>> {
    let id = Uuid::parse_str(&artist_id)?;
    let (page, limit) = get_pagination_params(album_filter);
    Ok(Json(
        db::get_albums_for_artist(&state.db, id, page, limit).await?,
    ))
}

fn get_pagination_params(album_filter: Query<AlbumFilter>) -> (i64, i64) {
    let limit = if album_filter.limit == 0 {
        25
    } else {
        album_filter.limit
    };
    let page = if album_filter.page == 0 {
        1
    } else {
        album_filter.page
    };
    (page, limit)
}
