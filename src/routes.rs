use axum::{
    Json,
    extract::{Path, Query, State},
};
use chrono::NaiveDate;
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    ApiContext, Result, db,
    types::{Album, NewAlbum},
};

#[derive(Deserialize, Clone)]
pub struct AlbumFilter {
    #[serde(default)]
    pub page: i64,
    #[serde(default)]
    pub limit: i64,

    pub genres: Option<String>,
    pub moods: Option<String>,
    pub min_rating: Option<f64>,
    pub since: Option<NaiveDate>,
    pub to: Option<NaiveDate>,
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
    Ok(Json(db::get_albums(&state.db, page, limit, &album_filter).await?))
}

pub async fn get_artist(
    State(state): State<ApiContext>,
    Path(artist_id): Path<String>,
    album_filter: Query<AlbumFilter>,
) -> Result<Json<Vec<Album>>> {
    let id = Uuid::parse_str(&artist_id)?;
    let (page, limit) = get_pagination_params(album_filter);
    Ok(Json(db::get_albums_for_artist(&state.db, id, page, limit).await?))
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
