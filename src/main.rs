mod controller;
mod error;
mod model;

use crate::controller::{index, stats, weather};
use anyhow::Context;
use axum::{routing::get, Router};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let db_connection_str = std::env::var("DATABASE_URL").context("DATABASE_URL must be set")?;

    let pool = sqlx::PgPool::connect(&db_connection_str)
        .await
        .context("can't connect to database")?;

    let app = Router::new()
        .route("/", get(index))
        .route("/weather", get(weather))
        .route("/stats", get(stats))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;

    axum::serve(listener, app).await?;

    Ok(())
}
