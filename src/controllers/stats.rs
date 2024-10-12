use crate::models::error::AppError;
use crate::models::stat::{City, StatsTemplate};
use axum::extract::State;
use sqlx::PgPool;

async fn get_last_cities(pool: &PgPool) -> Result<Vec<City>, AppError> {
    let cities = sqlx::query_as::<_, City>(
        "SELECT name, lat AS latitude, long AS longitude FROM cities ORDER BY id DESC LIMIT 10",
    )
    .fetch_all(pool)
    .await?;
    Ok(cities)
}

pub async fn stats(State(pool): State<PgPool>) -> Result<StatsTemplate, AppError> {
    let cities = get_last_cities(&pool).await?;
    Ok(StatsTemplate { cities })
}
