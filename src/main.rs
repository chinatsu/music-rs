use std::env;

use axum::{
    Router,
    routing::{get, post},
};
use axum_response_cache::CacheLayer;
use dotenv::dotenv;
use sqlx::{PgPool, postgres::PgPoolOptions};

mod db;
mod error;
mod routes;
mod types;

type Result<T> = std::result::Result<T, error::AppError>;

#[derive(Clone)]
struct ApiContext {
    pub db: PgPool,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();

    let db = PgPoolOptions::new()
        .max_connections(50)
        .connect(&env::var("DATABASE_URL")?)
        .await?;

    let router = Router::new()
        .route("/", get(routes::get_albums).layer(CacheLayer::with_lifespan(60)))
        .route("/update", post(routes::add_albums))
        .route("/date/{date}", get(routes::get_albums_for_date))
        .route("/genre/{genre}", get(routes::get_albums_for_genre))
        .route("/genre/similar/{genre}", get(routes::get_similar_genres))
        .route("/artist/{artist_id}", get(routes::get_artist))
        .with_state(ApiContext { db });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5000").await?;
    axum::serve(listener, router).await?;

    Ok(())
}
