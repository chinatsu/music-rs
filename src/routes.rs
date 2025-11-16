use axum::{
    Json,
    extract::{Path, Query, State},
};
use serde::Deserialize;
use time::{Date, format_description};
use uuid::Uuid;

use crate::{
    ApiContext, Result, db,
    types::{Album, GenreInfo, MoodInfo, NewAlbum},
};

#[derive(Deserialize)]
pub struct Pagination {
    #[serde(default)]
    page: i64,
    #[serde(default)]
    limit: i64,
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
    pagination: Query<Pagination>,
) -> Result<Json<Vec<Album>>> {
    let limit = if pagination.limit == 0 {
        100
    } else {
        pagination.limit
    };
    let page = if pagination.page == 0 {
        1
    } else {
        pagination.page
    };
    Ok(Json(db::get_albums(&state.db, page, limit).await?))
}

pub async fn get_artist(
    State(state): State<ApiContext>,
    Path(artist_id): Path<String>,
) -> Result<Json<Vec<Album>>> {
    let id = Uuid::parse_str(&artist_id)?;
    Ok(Json(db::get_albums_for_artist(&state.db, id).await?))
}

pub async fn get_albums_for_date(
    State(state): State<ApiContext>,
    Path(date): Path<String>,
) -> Result<Json<Vec<Album>>> {
    let format = format_description::parse("[year]-[month]-[day]")?;
    let parsed = Date::parse(&date, &format)?;
    Ok(Json(db::get_albums_for_date(&state.db, parsed).await?))
}

pub async fn get_genre(
    State(state): State<ApiContext>,
    Path(genre): Path<String>,
) -> Result<Json<GenreInfo>> {
    let genre_id = Uuid::parse_str(&genre)?;
    let db_genre = db::get_genre(&state.db, genre_id).await?;
    let db_similar_genres = db::get_similar_genres(&state.db, genre_id).await?;
    let db_genre_albums = db::get_albums_for_genre(&state.db, genre_id).await?;
    Ok(Json(GenreInfo {
        genre: db_genre,
        similar_genres: db_similar_genres,
        albums: db_genre_albums,
    }))
}

pub async fn get_mood(
    State(state): State<ApiContext>,
    Path(mood): Path<String>,
) -> Result<Json<MoodInfo>> {
    let mood_id = Uuid::parse_str(&mood)?;
    let db_mood = db::get_mood(&state.db, mood_id).await?;
    let db_similar_moods = db::get_similar_moods(&state.db, mood_id).await?;
    let db_mood_albums = db::get_albums_for_mood(&state.db, mood_id).await?;
    Ok(Json(MoodInfo {
        mood: db_mood,
        similar_moods: db_similar_moods,
        albums: db_mood_albums,
    }))
}
