use std::env;

use axum::{
    routing::{get, post}, Router
};
use dotenv::dotenv;
use sqlx::{PgPool, postgres::PgPoolOptions};

mod db;
mod routes;
mod types;
mod error;

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
        .route("/", get(routes::get_albums))
        .route("/", post(routes::add_albums))
        .route("/date/{date}", get(routes::get_albums_for_date))
        .route("/genre/{genres}", get(routes::get_albums_for_genre))
        .with_state(ApiContext { db });

    let listener = tokio::net::TcpListener::bind("0.0.0.0:5000").await?;
    axum::serve(listener, router).await?;

    Ok(())
}
